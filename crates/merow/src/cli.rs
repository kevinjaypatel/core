use camino::Utf8PathBuf;
use clap::Parser;
use const_format::concatcp;
// use tokio::process::Command;
use eyre::Result as EyreResult;
use std::{
    process::{Command, Output, Stdio},
    result,
};

pub const EXAMPLES: &str = r"
  # Start a new coordinator
  $ merow start-coordinator 

  # Start a new peer 
  $ merow start-peer 
";

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(after_help = concatcp!(
    "Examples:",
    EXAMPLES
))]
pub struct RootCommand {
    /// Name of the command
    pub action: String,
}

pub struct Node {
    name: String,
    node_type: Option<String>,
    home: Utf8PathBuf,
    server_port: u16,
    swarm_port: u16,
}
pub struct Peer {}

fn init_coordinator() -> EyreResult<()> {
    println!("Initializing coordinator...");

    // TODO: check if the data directory exists

    // Define the command to run the other binary package within the same workspace.
    // This example assumes the workspace has a binary package named `merod`.
    let output = Command::new("cargo")
        .arg("run") // The cargo run command
        .arg("-p") // Specify the package to run
        .arg("merod") // Name of the binary package in the workspace
        .arg("--") // Pass any arguments to the binary after this
        .arg("--node-name") // Example argument to the binary
        .arg("coordinator")
        .arg("--home")
        .arg("data")
        .arg("init")
        .arg("--server-port")
        .arg("2427")
        .arg("--swarm-port")
        .arg("2527")
        .stdout(Stdio::piped()) // Capture stdout
        .stderr(Stdio::piped()) // Capture stderr
        .output()?; // Execute the command and get the output

    println!("Status: {}", output.status);
    println!("Stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));

    Ok(()) // Return the output (stdout, stderr, and exit status)
}

async fn start_coordinator() -> EyreResult<()> {
    println!("Running coordinator...");
    let mut command = Command::new("cargo");
    command.arg("run"); // The cargo run command
    command.arg("-p"); // Specify the package to run
    command.arg("merod"); // Name of the binary package in the workspace
    command.arg("--"); // Pass any arguments to the binary after this
    command.arg("--node-name"); // Example argument to the binary
    command.arg("coordinator");
    command.arg("--home");
    command.arg("data");
    command.arg("run");
    command.stdin(Stdio::null());

    let child = command.spawn()?;
    // .arg("--node-type ")
    // .arg("coordinator")
    // .stdout(Stdio::piped()) // Capture stdout
    // .stderr(Stdio::piped()) // Capture stderr
    // .output()?; // Execute the command and get the output
    // .spawn()?; // Execute the command and get the output

    Ok(())
}

impl RootCommand {
    pub async fn run(self) -> EyreResult<()> {
        match self.action.as_str() {
            "init-coordinator" => {
                // TODO: check if coordinator is initialized
                init_coordinator()
            }
            "start-coordinator" => start_coordinator().await,
            "start-peer" => {
                // check if the peer is already running
                println!("Start peer was invoked...");
                Ok(())
            }
            _ => {
                println!("Unknown command...");
                Ok(())
            }
        }
    }
}
