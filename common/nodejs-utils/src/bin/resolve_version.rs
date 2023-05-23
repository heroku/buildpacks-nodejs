#![warn(clippy::pedantic)]

use heroku_nodejs_utils::{inv::Inventory, vrs::Requirement};

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

    let inv = Inventory::read(filename).unwrap_or_else(|e| {
        eprintln!("Error reading '{filename}': {e}");
        std::process::exit(INVENTORY_EXIT_CODE);
    });

    let version = inv.resolve(&version_requirements);
    if let Some(version) = version {
        println!("{} {}", version.version, version.url);
    } else {
        println!("No result");
    }
}
