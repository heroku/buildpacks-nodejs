#![warn(clippy::pedantic)]

use heroku_nodejs_utils::{
    distribution::{Distribution, DEFAULT_BUCKET},
    inv::Inventory,
};
use std::str::FromStr;

const FAILED_EXIT_CODE: i32 = 1;

/// Prints a human-readable software inventory difference. Useful
/// for generating commit messages and changelogs for automated inventory
/// updates.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        print_usage();
        std::process::exit(FAILED_EXIT_CODE);
    }

    let distribution = Distribution::from_str(&args[1]).unwrap_or_else(|e| {
        eprintln!("Error reading distribution: {e}");
        print_usage();
        std::process::exit(1);
    });
    let inventory_loc = &args[2];

    let bucket = std::env::var("AWS_S3_BUCKET").unwrap_or(DEFAULT_BUCKET.to_string());

    let mirrored_versions = distribution.mirrored_versions(&bucket).unwrap_or_else(|e| {
        eprintln!("Error reading mirrored versions: {e}");
        std::process::exit(1);
    });

    let local_versions = Inventory::read(inventory_loc)
        .unwrap_or_else(|e| {
            eprintln!("Error reading '{inventory_loc}': {e}");
            std::process::exit(1);
        })
        .releases
        .iter()
        .map(|r| r.version.clone())
        .collect();

    let msg = [
        ("Added", mirrored_versions.difference(&local_versions)),
        ("Removed", local_versions.difference(&mirrored_versions)),
    ]
    .iter()
    .filter(|(_, versions)| versions.clone().count() > 0)
    .flat_map(|(change, versions)| {
        versions
            .clone()
            .map(|version| format!("- {} {} version {}.", *change, distribution, version))
    })
    .collect::<Vec<String>>()
    .join("\n");

    println!("{msg}");
}

fn print_usage() {
    eprintln!("$ AWS_S3_BUCKET=heroku-nodebin diff_versions <node|yarn> path/to/inventory.toml");
}
