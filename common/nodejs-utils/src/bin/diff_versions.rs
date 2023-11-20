// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use heroku_nodejs_utils::vrs::Version;
use heroku_nodejs_utils::{
    distribution::{Distribution, DEFAULT_BUCKET},
    inv::Inventory,
};
use std::collections::HashSet;
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

    let added_versions = list_version_differences(&mirrored_versions, &local_versions)
        .into_iter()
        .map(|version| format!("- Added {distribution} version {version}."))
        .collect::<Vec<_>>();

    if !added_versions.is_empty() {
        println!("{}", added_versions.join("\n"));
    }

    let removed_versions = list_version_differences(&local_versions, &mirrored_versions)
        .into_iter()
        .map(|version| format!("- Removed {distribution} version {version}."))
        .collect::<Vec<_>>();

    if !removed_versions.is_empty() {
        println!("{}", removed_versions.join("\n"));
    }
}

fn list_version_differences(
    versions_a: &HashSet<Version>,
    versions_b: &HashSet<Version>,
) -> Vec<Version> {
    let mut differences = versions_a
        .difference(versions_b)
        .cloned()
        .collect::<Vec<_>>();
    differences.sort();
    differences
}

fn print_usage() {
    eprintln!(
        "$ AWS_S3_BUCKET=heroku-nodebin diff_versions <node|yarn|npm> path/to/inventory.toml"
    );
}
