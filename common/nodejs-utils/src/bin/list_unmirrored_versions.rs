#![warn(clippy::pedantic)]

use heroku_nodejs_utils::distribution::Distribution;

fn main() {
    let dist = std::env::args()
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
        });

    let upstream_versions = dist.upstream_versions().unwrap_or_else(|e| {
        eprintln!("Couldn't fetch upstream version list: {e}");
        std::process::exit(1);
    });

    let mirrored_versions = dist.mirrored_versions().unwrap_or_else(|e| {
        eprintln!("Couldn't fetch mirrored version list: {e}");
        std::process::exit(1);
    });

    let unmirrored_versions = upstream_versions.difference(&mirrored_versions);
    for version in dist
        .filter_inactive_versions(unmirrored_versions)
        .unwrap_or_else(|e| {
            eprintln!("Error filtering inactive versions: {e}");
            std::process::exit(1);
        })
    {
        println!("{version}");
    }
}

fn print_usage() {
    eprintln!("Usage: $ list_unmirrored_versions <node|yarn>");
}
