// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use std::env::consts;
use sha2::Sha256;

use heroku_nodejs_utils::vrs::{Requirement, Version};
use heroku_inventory_utils::inv::{read_inventory_file, resolve, Arch, Os, Inventory};
use heroku_nodejs_utils::inv::Inventory as NodeJSInventory;

const SUCCESS_EXIT_CODE: i32 = 0;
const ARGS_EXIT_CODE: i32 = 1;
const VERSION_REQS_EXIT_CODE: i32 = 2;
const INVENTORY_EXIT_CODE: i32 = 3;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 0 && (&args[1] == "-v" || &args[1] == "--version") {
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

    // Try to parse the new inventory file format first
    let inv: Option<Inventory<Version, Sha256>> = read_inventory_file(&filename).ok();

    // Fall back to the old format. Remove this after cutting over to the new format.
    if inv.is_none() {
        let inv = NodeJSInventory::read(filename).unwrap_or_else(|e| {
            eprintln!("Error reading '{filename}': {e}");
            std::process::exit(INVENTORY_EXIT_CODE);
        });

        let version = inv.resolve(&version_requirements);
        if let Some(version) = version {
            println!("{} {}", version.version, version.url);
        } else {
            eprintln!("No result");
        }
        std::process::exit(SUCCESS_EXIT_CODE);
    }

    let inv = inv.unwrap();
    let artifact = match (consts::OS.parse::<Os>(), consts::ARCH.parse::<Arch>()) {
        (Ok(os), Ok(arch)) => resolve(&inv.artifacts, os, arch, &version_requirements),
        (_, _) => None,
    }.unwrap_or_else(|| {
        eprintln!("Could not find version to satisfy requirements \"{}\" for OS {} on arch {}", version_requirements.to_string(), consts::OS.to_string(), consts::ARCH.to_string());
        std::process::exit(VERSION_REQS_EXIT_CODE);
    });

    println!("{} {}", artifact.version, artifact.url);
}
