use crate::verify_node::{VerifyNodeArgs, verify_node};
use clap::Parser;

mod support;
mod verify_node;

#[tokio::main]
async fn main() {
    match Cli::parse() {
        Cli::VerifyNode(args) => verify_node(&args).await,
    };
}

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
enum Cli {
    VerifyNode(VerifyNodeArgs),
}
