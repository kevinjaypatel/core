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

#[derive(Serialize, Deserialize, Debug)]
struct NodeData {
    coordinator: NodeConfig,
    admin: NodeConfig,
}

#[derive(Serialize, Deserialize, Debug)]
struct NodeConfig {
    name: String,
    server_port: u16,
    swarm_port: u16,
    home: String,
}

fn init_coordinator(data: NodeData) -> EyreResult<()> {
    println!("Initializing coordinator...");

    // Sets the default configuration for the node
    let node_name: &OsStr = OsStr::new(&data.coordinator.name);
    let node_home: &OsStr = OsStr::new(&data.coordinator.home);

    let server_port_str = data.coordinator.server_port.to_string();
    let swarm_port_str = data.coordinator.swarm_port.to_string();
    let server_port: &OsStr = OsStr::new(&server_port_str);
    let swarm_port = OsStr::new(&swarm_port_str);

    // create the home directory if it doesnt exist
    if !Path::new(node_home).is_dir() {
        // Make the Node home directory
        let result = match fs::create_dir(node_home) {
            Ok(()) => match node_home.to_str() {
                Some(valid_str) => println!("Created Node Home Directory: {}", valid_str),
                None => println!("OsStr contains non-UTF-8 data: {:?}", node_home),
            },
            Err(error) => panic!("Problem creating the Node Home directory: {error:?}"),
        };
    }

    // Define the command to run the other binary package within the same workspace.
    // This example assumes the workspace has a binary package named `merod`.
    let mut command = Command::new("cargo");

    command.arg("run"); // The cargo run command
    command.arg("-p"); // Specify the package to run
    command.arg("merod"); // Name of the binary package in the workspace
    command.arg("--"); // Pass any arguments to the binary after this
    command.arg("--node-name"); // Example argument to the binary
    command.arg(node_name);
    command.arg("--home");
    command.arg(node_home);
    command.arg("init");
    command.arg("--server-port");
    command.arg(server_port);
    command.arg("--swarm-port");
    command.arg(swarm_port);
    command.stdout(Stdio::piped()); // Capture stdout
    command.stderr(Stdio::piped()); // Capture stderr

    let child = command.output()?; // Execute the command and get the output
    println!("Status: {}", child.status);
    println!("Stdout: {}", String::from_utf8_lossy(&child.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&child.stderr));

    Ok(()) // Return the output (stdout, stderr, and exit status)
}

fn init_node() -> EyreResult<()> {
    println!("Initializing node...");

    // TODO: check if the data directory exists

    // Define the command to run the other binary package within the same workspace.
    // This example assumes the workspace has a binary package named `merod`.
    let mut command = Command::new("cargo");

    command.arg("run"); // The cargo run command
    command.arg("-p"); // Specify the package to run
    command.arg("merod"); // Name of the binary package in the workspace
    command.arg("--"); // Pass any arguments to the binary after this
    command.arg("--node-name"); // Example argument to the binary
    command.arg("node");
    command.arg("--home");
    command.arg("data");
    command.arg("init");
    command.arg("--server-port");
    command.arg("2428");
    command.arg("--swarm-port");
    command.arg("2528");
    command.stdout(Stdio::piped()); // Capture stdout
    command.stderr(Stdio::piped()); // Capture stderr

    let child = command.output()?; // Execute the command and get the output
    println!("Status: {}", child.status);
    println!("Stdout: {}", String::from_utf8_lossy(&child.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&child.stderr));

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
    // command.stdin(Stdio::null());

    // let child = command.spawn()?;
    let child = command.output()?;

    Ok(())
}

async fn start_node() -> EyreResult<()> {
    println!("Running node...");

    let mut command = Command::new("cargo");

    command.arg("run"); // The cargo run command
    command.arg("-p"); // Specify the package to run
    command.arg("merod"); // Name of the binary package in the workspace
    command.arg("--"); // Pass any arguments to the binary after this
    command.arg("--node-name"); // Example argument to the binary
    command.arg("node1");
    command.arg("--home");
    command.arg("data");
    command.arg("run");
    // command.stdin(Stdio::null());

    // let child = command.spawn()?;

    // Execute the command as a child process
    let child = command.output()?;

    // Display the captured output from the child
    println!("Status: {}", child.status);
    println!("Stdout: {}", String::from_utf8_lossy(&child.stdout));
    println!("Stderr: {}", String::from_utf8_lossy(&child.stderr));

    Ok(())
}

impl RootCommand {
    pub async fn run(self) -> EyreResult<()> {
        match self.action.as_str() {
            "init-coordinator" => {
                let data = NodeData::get_noad_data();

                // TODO: check if coordinator is initialized @data.coordinator.home
                init_coordinator(data)
            }
            "init-node" => init_node(),
            "start-coordinator" => start_coordinator().await,
            "start-node" => start_node().await,
            _ => {
                println!("Unknown command...");
                Ok(())
            }
        }
    }
}

impl NodeData {
    fn get_noad_data() -> NodeData {
        // TODO: Make Node Configuration Filepath constant with global scope
        let filename = "crates/merow/config/default.toml";

        let contents = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(_) => {
                eprintln!("Could not read file `{}`", filename);
                exit(1);
            }
        };

        println!("TOML Contents: \n{}", contents);

        let node_data: NodeData = match toml::from_str(&contents) {
            Ok(nd) => nd,
            Err(_) => {
                // Write `msg` to `stderr`.
                eprintln!("Unable to load data from `{}`", filename);
                // Exit the program with exit code `1`.
                exit(1);
            }
        };

        // Convert the NodeData to a JSON string
        // let serialized = serde_json::to_string(&node_data).unwrap();
        // println!("serialized = {}", serialized);

        return node_data;
    }
}
