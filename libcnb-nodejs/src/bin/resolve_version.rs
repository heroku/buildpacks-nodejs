use libcnb_nodejs::versions::{Inventory, Req};

const SUCCESS_EXIT_CODE: i32 = 0;
const ARGS_EXIT_CODE: i32 = 1;
const VERSION_REQS_EXIT_CODE: i32 = 2;
const IO_EXIT_CODE: i32 = 3;
const TOML_EXIT_CODE: i32 = 4;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if &args[1] == "-v" || &args[1] == "--version" {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("v{}", VERSION);
        std::process::exit(SUCCESS_EXIT_CODE);
    }

    if args.len() < 3 {
        eprintln!("$ resolve <toml file> <version requirements>");
        std::process::exit(ARGS_EXIT_CODE);
    }
    let filename = &args[1];
    let version_requirements = Req::parse(&args[2]).unwrap_or_else(|e| {
        eprintln!("Could not parse Version Requirements '{}': {}", &args[2], e);
        std::process::exit(VERSION_REQS_EXIT_CODE);
    });

    let contents = std::fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Could not read file '{}': {}", filename, e);
        std::process::exit(IO_EXIT_CODE);
    });
    let inv: Inventory = toml::from_str(&contents).unwrap_or_else(|e| {
        eprintln!("Could not parse toml of '{}': {}", filename, e);
        std::process::exit(TOML_EXIT_CODE);
    });

    let version = inv.resolve(version_requirements);
    if let Some(version) = version {
        println!("{} {}", version.version, version.url);
    } else {
        println!("No result");
    }
}
