use clap::Parser; 
use const_format::concatcp;

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

impl RootCommand {
  pub fn run(self) {
      match self.action.as_str() {
          "start-coordinator" => println!("Start coordinator was invoked..."), 
          "start-peer" => println!("Start peer was invoked..."),
          _ => println!("Unknown command..."),
      }
  }
}