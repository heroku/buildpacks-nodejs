// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use clap::{arg, value_parser, Command};
use heroku_nodejs_utils::vrs::Requirement;
use heroku_nodejs_utils::vrs::Version;
use libherokubuildpack::inventory::artifact::{Arch, Os};
use sha2::Sha256;
use std::env::consts;

const VERSION_REQS_EXIT_CODE: i32 = 1;
const INVENTORY_EXIT_CODE: i32 = 2;
const UNSUPPORTED_OS_EXIT_CODE: i32 = 3;
const UNSUPPORTED_ARCH_EXIT_CODE: i32 = 4;

fn main() {
    let matches = Command::new("resolve_version")
        .arg(arg!(<inventory_path>))
        .arg(arg!(<node_version>))
        .arg(arg!(--os <os>).value_parser(value_parser!(Os)))
        .arg(arg!(--arch <arch>).value_parser(value_parser!(Arch)))
        .get_matches();

    let inventory_path = matches
        .get_one::<String>("inventory_path")
        .expect("required argument");
    let node_version = matches
        .get_one::<String>("node_version")
        .expect("required argument");
    let os = match matches.get_one::<Os>("os") {
        Some(os) => *os,
        None => consts::OS.parse::<Os>().unwrap_or_else(|e| {
            eprintln!("Unsupported OS '{}': {e}", consts::OS);
            std::process::exit(UNSUPPORTED_OS_EXIT_CODE);
        }),
    };
    let arch = match matches.get_one::<Arch>("arch") {
        Some(arch) => *arch,
        None => consts::ARCH.parse::<Arch>().unwrap_or_else(|e| {
            eprintln!("Unsupported Architecture '{}': {e}", consts::ARCH);
            std::process::exit(UNSUPPORTED_ARCH_EXIT_CODE);
        }),
    };

    let version_requirements = Requirement::parse(node_version).unwrap_or_else(|e| {
        eprintln!("Could not parse Version Requirements '{node_version}': {e}");
        std::process::exit(VERSION_REQS_EXIT_CODE);
    });

    let inv_contents = std::fs::read_to_string(inventory_path).unwrap_or_else(|e| {
        eprintln!("Error reading '{inventory_path}': {e}");
        std::process::exit(INVENTORY_EXIT_CODE);
    });

    let inv: libherokubuildpack::inventory::Inventory<Version, Sha256, Option<()>> =
        toml::from_str(&inv_contents).unwrap_or_else(|e| {
            eprintln!("Error parsing '{inventory_path}': {e}");
            std::process::exit(INVENTORY_EXIT_CODE);
        });

    let version = inv.resolve(os, arch, &version_requirements);

    if let Some(version) = version {
        println!(
            "{} {} {} {}",
            version.version,
            version.url,
            version.checksum.name,
            hex::encode(&version.checksum.value)
        );
    } else {
        println!("No result");
    }
}
