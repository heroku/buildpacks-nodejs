use crate::download_verify_node::{download_verify_node, download_verify_node_cmd};
use clap::command;

mod download_verify_node;
mod download_verify_npm_package;
mod support;

#[tokio::main]
async fn main() {
    let download_verify_node_cmd = download_verify_node_cmd();

    let matches = command!()
        .disable_version_flag(true)
        .subcommand(&download_verify_node_cmd)
        .get_matches();

    if let Some(args) = matches.subcommand_matches(download_verify_node_cmd.get_name()) {
        download_verify_node(args).await;
    }
}
