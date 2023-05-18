#![warn(clippy::pedantic)]

use heroku_nodejs_utils::distribution::Distribution;
use heroku_nodejs_utils::inv::BUCKET;
use heroku_nodejs_utils::vrs::Version;

/// This command prints a list of Yarn or Node.js versions that have not
/// yet been published to Nodebin (S3 bucket). It checks the list of upstream
/// releases to what is listed in the AWS_S3_BUCKET. Output is a JSON array,
/// so that it may be parsed by GitHub actions.
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

    let bucket = std::env::var("AWS_S3_BUCKET").unwrap_or(BUCKET.to_string());

    // Limit the number of versions returned. This is useful to prevent
    // spawning 100+ GitHub actions when querying against an empty S3 bucket.
    let version_limit: usize = std::env::var("VERSION_LIMIT")
        .unwrap_or("12".to_string())
        .parse()
        .unwrap_or_else(|e| {
            eprintln!("Couldn't parse version limit: {e}");
            std::process::exit(1);
        });

    let upstream_versions = dist.upstream_versions().unwrap_or_else(|e| {
        eprintln!("Couldn't fetch upstream version list: {e}");
        std::process::exit(1);
    });

    let mirrored_versions = dist.mirrored_versions(&bucket).unwrap_or_else(|e| {
        eprintln!("Couldn't fetch mirrored version list: {e}");
        std::process::exit(1);
    });

    let unmirrored_versions: Vec<String> = dist
        .filter_inactive_versions(upstream_versions.difference(&mirrored_versions))
        .unwrap_or_else(|e| {
            eprintln!("Error filtering inactive versions: {e}");
            std::process::exit(1);
        })
        .iter()
        .take(version_limit)
        .map(Version::to_string)
        .collect();

    serde_json::to_writer(std::io::stdout(), &unmirrored_versions).unwrap_or_else(|e| {
        eprintln!("Couldn't write versions to stdout: {e}");
        std::process::exit(1);
    });
}

fn print_usage() {
    eprintln!("Usage: $ AWS_S3_BUCKET=heroku-nodebin list_unmirrored_versions <node|yarn>");
}
