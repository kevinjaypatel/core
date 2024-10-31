# Core Cli Wrapper (merow) 

A CLI wrapper for the Calimero Node that provides a default node configuration file for initializing a node, and quickly running a development environment for testing P2P Calimero Apps. 

## Features

- Custom Node Configuration File 
- Simple Commands to Initialize and Run a Calimero Node 
- Creates a Node Home Directory (if it doesn't already exist)




## Usage

Setup the Default Configuration:  `config/default.toml` 

```javascript
[coordinator]
name = "coordinator" 
server_port = 2427
swarm_port = 2527
home = "data"

[admin] 
name = "node1" 
server_port = 2428 
swarm_port = 2528
home = "data"
```

Initialize a coordinator   
`$ merow -- init-coordinator` 

Initialize a node   
`$ merow -- init-node` 

Start a running coordinator   
`$ merow -- start-coordinator` 

Start a running node   
`$ merow -- start-node` 


 

## Roadmap

- Additional commands for `Dev Context` creation and `Peer Invitation`
- Add a boolean flag to the Configuration File for Deploying the Admin Dashboard 
- Multi-node deployment (e.g. node1, node2, ... nodeN)
