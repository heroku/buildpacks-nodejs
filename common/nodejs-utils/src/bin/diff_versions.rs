#![warn(clippy::pedantic)]

use heroku_nodejs_utils::inv::{Inventory, BUCKET, REGION};
use heroku_nodejs_utils::nodebin_s3;
use std::collections::HashSet;
use std::convert::TryFrom;

const FAILED_EXIT_CODE: i32 = 1;

/// Prints a human-readable software inventory difference. Useful
/// for generating commit messages and changelogs for automated inventory
/// updates.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("$ list_versions <node|yarn> path/to/inventory.toml");
        std::process::exit(FAILED_EXIT_CODE);
    }

    let software_name = &args[1];
    let inventory_loc = &args[2];

    let remote_objects =
        nodebin_s3::list_objects(BUCKET, REGION, software_name).unwrap_or_else(|e| {
            eprintln!("Failed to fetch from S3: {}", e);
            std::process::exit(1);
        });

    let remote_versions: HashSet<String> = Inventory::try_from(remote_objects)
        .unwrap_or_else(|e| {
            eprintln!("Failed to parse AWS S3 XML: {}", e);
            std::process::exit(2);
        })
        .releases
        .iter()
        .map(|r| r.version.to_string())
        .collect();

    let local_versions: HashSet<String> = Inventory::read(inventory_loc)
        .unwrap_or_else(|e| {
            eprintln!("Error reading '{}': {}", inventory_loc, e);
            std::process::exit(3);
        })
        .releases
        .iter()
        .map(|r| r.version.to_string())
        .collect();

    let msg = [
        ("Added", remote_versions.difference(&local_versions)),
        ("Removed", local_versions.difference(&remote_versions)),
    ]
    .iter()
    .filter_map(|(change, versions)| {
        if versions.clone().count() > 0 {
            Some(format!(
                "{} {} version {}.",
                change,
                software_name,
                versions
                    .clone()
                    .map(|v| v.to_owned())
                    .collect::<Vec<String>>()
                    .join(", ")
            ))
        } else {
            None
        }
    })
    .collect::<Vec<String>>()
    .join(" ");

    println!("{}", msg);
}
