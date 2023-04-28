#![warn(clippy::pedantic)]

use heroku_nodejs_utils::distribution::Distribution;

fn main() {
    let inventory = std::env::args()
        .nth(1)
        .unwrap_or_else(|| {
            eprintln!("Missing distribution argument.");
            print_usage();
            std::process::exit(1);
        })
        .parse::<Distribution>()
        .unwrap_or_else(|e| {
            eprintln!("Unknown distribution: {e}");
            print_usage();
            std::process::exit(1);
        })
        .mirrored_inventory()
        .unwrap_or_else(|e| {
            eprintln!("Failed to read mirrored releases: {e}");
            print_usage();
            std::process::exit(1);
        });

    let output = toml::to_string(&inventory).unwrap_or_else(|e| {
        eprintln!("Failed to serialize inventory as toml: {e}");
        print_usage();
        std::process::exit(1);
    });

    println!("{output}");
}

fn print_usage() {
    eprintln!("Usage: $ generate_inventory <node|yarn>");
}
