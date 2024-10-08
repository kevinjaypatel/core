#![allow(clippy::exhaustive_structs, reason = "TODO: Allowed until reviewed")]

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time;

pub use near_crypto::SecretKey;
use near_crypto::{InMemorySigner, Signer};
use near_jsonrpc_client::methods::query::{RpcQueryRequest, RpcQueryResponse};
use near_jsonrpc_client::methods::send_tx::RpcSendTransactionRequest;
use near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest;
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_primitives::types::query::QueryResponseKind;
use near_jsonrpc_primitives::types::transactions::{RpcTransactionError, TransactionInfo};
use near_primitives::account::id::ParseAccountError;
use near_primitives::action::{Action, FunctionCallAction};
use near_primitives::hash::CryptoHash;
use near_primitives::transaction::{Transaction, TransactionV0};
pub use near_primitives::types::AccountId;
use near_primitives::types::{BlockReference, FunctionArgs};
use near_primitives::views::{
    AccessKeyPermissionView, AccessKeyView, CallResult, FinalExecutionStatus, QueryRequest,
    TxExecutionStatus,
};
use thiserror::Error;
use url::Url;

use super::{Operation, Transport, TransportRequest};

#[derive(Debug)]
pub struct NetworkConfig {
    pub rpc_url: Url,
    pub account_id: AccountId,
    pub access_key: SecretKey,
}

#[derive(Debug)]
pub struct NearConfig<'a> {
    pub networks: BTreeMap<Cow<'a, str>, NetworkConfig>,
}

#[derive(Clone, Debug)]
struct Network {
    client: JsonRpcClient,
    account_id: AccountId,
    secret_key: SecretKey,
}

#[derive(Clone, Debug)]
pub struct NearTransport<'a> {
    networks: BTreeMap<Cow<'a, str>, Network>,
}

impl<'a> NearTransport<'a> {
    #[must_use]
    pub fn new(config: &NearConfig<'a>) -> Self {
        let mut networks = BTreeMap::new();

        for (network_id, network_config) in &config.networks {
            let client = JsonRpcClient::connect(network_config.rpc_url.clone());

            let _ignored = networks.insert(
                network_id.clone(),
                Network {
                    client,
                    account_id: network_config.account_id.clone(),
                    secret_key: network_config.access_key.clone(),
                },
            );
        }

        Self { networks }
    }
}

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum NearError {
    #[error("unknown network `{0}`")]
    UnknownNetwork(String),
    #[error("invalid response from RPC while {operation}")]
    InvalidResponse { operation: ErrorOperation },
    #[error("invalid contract ID `{0}`")]
    InvalidContractId(ParseAccountError),
    #[error("access key does not have permission to call contract `{0}`")]
    NotPermittedToCallContract(AccountId),
    #[error(
        "access key does not have permission to call method `{method}` on contract {contract}"
    )]
    NotPermittedToCallMethod { contract: AccountId, method: String },
    #[error("transaction timed out")]
    TransactionTimeout,
    #[error("error while {operation}: {reason}")]
    Custom {
        operation: ErrorOperation,
        reason: String,
    },
}

#[derive(Copy, Clone, Debug, Error)]
#[non_exhaustive]
pub enum ErrorOperation {
    #[error("querying contract")]
    Query,
    #[error("mutating contract")]
    Mutate,
    #[error("fetching account")]
    FetchAccount,
}

impl Transport for NearTransport<'_> {
    type Error = NearError;

    async fn send(
        &self,
        request: TransportRequest<'_>,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, Self::Error> {
        let Some(network) = self.networks.get(&request.network_id) else {
            return Err(NearError::UnknownNetwork(request.network_id.into_owned()));
        };

        let contract_id = request
            .contract_id
            .parse()
            .map_err(NearError::InvalidContractId)?;

        match request.operation {
            Operation::Read { method } => {
                network
                    .query(contract_id, method.into_owned(), payload)
                    .await
            }
            Operation::Write { method } => {
                network
                    .mutate(contract_id, method.into_owned(), payload)
                    .await
            }
        }
    }
}

impl Network {
    async fn query(
        &self,
        contract_id: AccountId,
        method: String,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, NearError> {
        let response = self
            .client
            .call(RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: QueryRequest::CallFunction {
                    account_id: contract_id,
                    method_name: method,
                    args: FunctionArgs::from(args),
                },
            })
            .await
            .map_err(|err| NearError::Custom {
                operation: ErrorOperation::Query,
                reason: err.to_string(),
            })?;

        #[expect(clippy::wildcard_enum_match_arm, reason = "This is reasonable here")]
        match response.kind {
            QueryResponseKind::CallResult(CallResult { result, .. }) => Ok(result),
            _ => Err(NearError::InvalidResponse {
                operation: ErrorOperation::Query,
            }),
        }
    }

    async fn mutate(
        &self,
        contract_id: AccountId,
        method: String,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, NearError> {
        let (nonce, block_hash) = self.get_nonce(contract_id.clone(), method.clone()).await?;

        let transaction = Transaction::V0(TransactionV0 {
            signer_id: self.account_id.clone(),
            public_key: self.secret_key.public_key(),
            nonce: nonce.saturating_add(1),
            receiver_id: contract_id,
            block_hash,
            actions: vec![Action::FunctionCall(Box::new(FunctionCallAction {
                method_name: method,
                args,
                gas: 100_000_000_000_000, // 100 TeraGas
                deposit: 0,
            }))],
        });

        let (tx_hash, _) = transaction.get_hash_and_size();

        let sent_at = time::Instant::now();

        let mut response = self
            .client
            .call(RpcSendTransactionRequest {
                signed_transaction: transaction.sign(&Signer::InMemory(
                    InMemorySigner::from_secret_key(
                        self.account_id.clone(),
                        self.secret_key.clone(),
                    ),
                )),
                wait_until: TxExecutionStatus::Final,
            })
            .await;

        let response = loop {
            match response {
                Ok(response) => break response,
                Err(err) => {
                    let Some(RpcTransactionError::TimeoutError) = err.handler_error() else {
                        return Err(NearError::Custom {
                            operation: ErrorOperation::Mutate,
                            reason: err.to_string(),
                        });
                    };

                    if sent_at.elapsed().as_secs() > 60 {
                        return Err(NearError::TransactionTimeout);
                    }

                    response = self
                        .client
                        .call(RpcTransactionStatusRequest {
                            transaction_info: TransactionInfo::TransactionId {
                                tx_hash,
                                sender_account_id: self.account_id.clone(),
                            },
                            wait_until: TxExecutionStatus::Final,
                        })
                        .await;
                }
            }
        };

        let Some(outcome) = response.final_execution_outcome else {
            return Err(NearError::InvalidResponse {
                operation: ErrorOperation::Mutate,
            });
        };

        match outcome.into_outcome().status {
            FinalExecutionStatus::SuccessValue(value) => Ok(value),
            FinalExecutionStatus::Failure(error) => Err(NearError::Custom {
                operation: ErrorOperation::Mutate,
                reason: error.to_string(),
            }),
            FinalExecutionStatus::NotStarted | FinalExecutionStatus::Started => {
                Err(NearError::InvalidResponse {
                    operation: ErrorOperation::Mutate,
                })
            }
        }
    }

    async fn get_nonce(
        &self,
        contract_id: AccountId,
        method: String,
    ) -> Result<(u64, CryptoHash), NearError> {
        let response = self
            .client
            .call(RpcQueryRequest {
                block_reference: BlockReference::latest(),
                request: QueryRequest::ViewAccessKey {
                    account_id: self.account_id.clone(),
                    public_key: self.secret_key.public_key().clone(),
                },
            })
            .await
            .map_err(|err| NearError::Custom {
                operation: ErrorOperation::FetchAccount,
                reason: err.to_string(),
            })?;

        let RpcQueryResponse {
            kind: QueryResponseKind::AccessKey(AccessKeyView { nonce, permission }),
            block_hash,
            ..
        } = response
        else {
            return Err(NearError::InvalidResponse {
                operation: ErrorOperation::FetchAccount,
            });
        };

        if let AccessKeyPermissionView::FunctionCall {
            receiver_id,
            method_names,
            ..
        } = permission
        {
            if receiver_id != contract_id {
                return Err(NearError::NotPermittedToCallContract(contract_id));
            }

            if !(method_names.is_empty() || method_names.contains(&method)) {
                return Err(NearError::NotPermittedToCallMethod {
                    contract: contract_id,
                    method,
                });
            }
        }

        Ok((nonce, block_hash))
    }
}
