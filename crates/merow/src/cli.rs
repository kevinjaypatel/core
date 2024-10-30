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

fn init_coordinator() -> EyreResult<()> {
    println!("Initializing coordinator...");

    // TODO: check if the data directory exists

    // Define the command to run the other binary package within the same workspace.
    // This example assumes the workspace has a binary package named `merod`.
    let mut command = Command::new("cargo");

    command.arg("run"); // The cargo run command
    command.arg("-p"); // Specify the package to run
    command.arg("merod"); // Name of the binary package in the workspace
    command.arg("--"); // Pass any arguments to the binary after this
    command.arg("--node-name"); // Example argument to the binary
    command.arg("coordinator");
    command.arg("--home");
    command.arg("data");
    command.arg("init");
    command.arg("--server-port");
    command.arg("2427");
    command.arg("--swarm-port");
    command.arg("2527");
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
                let data = NodeData::load_data();

                // TODO: check if coordinator is initialized
                init_coordinator()
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
    fn load_data() {
        let path = match env::current_dir() {
            Ok(path) => println!("Current working directory: {}", path.display()),
            Err(e) => eprintln!("Failed to get current directory: {}", e),
        };

        let filename = "crates/merow/config/default.toml";

        let contents = match fs::read_to_string(filename) {
            // If successful return the files text as `contents`.
            // `c` is a local variable.
            Ok(c) => c,
            // Handle the `error` case.
            Err(_) => {
                // Write `msg` to `stderr`.
                eprintln!("Could not read file `{}`", filename);
                // Exit the program with exit code `1`.
                exit(1);
            }
        };

        println!("TOML Contents: \n{}", contents);
    }
}
