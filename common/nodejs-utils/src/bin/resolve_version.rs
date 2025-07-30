// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use heroku_nodejs_utils::vrs::Requirement;
use heroku_nodejs_utils::vrs::Version;
use libherokubuildpack::inventory::artifact::{Arch, Os};
use sha2::{Digest, Sha256};
use std::env::consts;

const SUCCESS_EXIT_CODE: i32 = 0;
const ARGS_EXIT_CODE: i32 = 1;
const VERSION_REQS_EXIT_CODE: i32 = 2;
const INVENTORY_EXIT_CODE: i32 = 3;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if &args[1] == "-v" || &args[1] == "--version" {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("v{VERSION}");
        std::process::exit(SUCCESS_EXIT_CODE);
    }

    if args.len() < 3 {
        eprintln!("$ resolve <toml file> <version requirements>");
        std::process::exit(ARGS_EXIT_CODE);
    }

    let filename = &args[1];

    let version_requirements = Requirement::parse(&args[2]).unwrap_or_else(|e| {
        eprintln!("Could not parse Version Requirements '{}': {e}", &args[2]);
        std::process::exit(VERSION_REQS_EXIT_CODE);
    });

    let inv_contents = std::fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Error reading '{filename}': {e}");
        std::process::exit(INVENTORY_EXIT_CODE);
    });

    let inv: libherokubuildpack::inventory::Inventory<Version, Sha256, Option<()>> =
        toml::from_str(&inv_contents).unwrap_or_else(|e| {
            eprintln!("Error parsing '{filename}': {e}");
            std::process::exit(INVENTORY_EXIT_CODE);
        });

    let version = match (consts::OS.parse::<Os>(), consts::ARCH.parse::<Arch>()) {
        (Ok(os), Ok(arch)) => inv.resolve(os, arch, &version_requirements),
        (_, _) => None,
    };

    if let Some(version) = version {
        println!(
            "{} {} {} {:x}",
            version.version,
            version.url,
            version.checksum.name,
            Sha256::digest(&version.checksum.value)
        );
    } else {
        println!("No result");
    }
}
