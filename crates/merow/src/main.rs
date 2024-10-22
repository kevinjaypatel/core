use clap::Parser;
use crate::cli::RootCommand;

mod cli; 

fn main() {
    let command = RootCommand::parse();
    command.run();   
}