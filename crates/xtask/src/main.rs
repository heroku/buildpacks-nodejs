use crate::download_verify_node::{DownloadVerifyNodeCommandArgs, execute_download_verify_node};
use clap::Parser;

mod download_verify_node;
mod support;

#[tokio::main]
async fn main() {
    match Cli::parse() {
        Cli::DownloadVerifyNode(args) => {
            execute_download_verify_node(&args).await;
        }
    };
}

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
enum Cli {
    DownloadVerifyNode(DownloadVerifyNodeCommandArgs),
}
