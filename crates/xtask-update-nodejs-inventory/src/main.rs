//! Updates the local node.js inventory.toml with versions published on nodejs.org.

mod node_releases;
mod output;
mod trusted_release_keys;

use clap::{ArgAction, ValueEnum, arg, command, value_parser};
use libherokubuildpack::inventory::artifact::{Arch, Os};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(
            arg!(<inventory_path>)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(
            arg!(<changelog_path>)
                .value_parser(value_parser!(PathBuf))
                .required(true),
        )
        .arg(
            arg!(--platform <platform>)
                .action(ArgAction::Append)
                .value_parser(value_parser!(SupportedNodeReleasePlatform))
                .default_values(["linux-x64", "linux-arm64"]),
        )
        .arg(
            arg!(--format <format>)
                .value_parser(value_parser!(OutputFormat))
                .default_value("keep-a-changelog"),
        )
        .get_matches();

    let inventory_path = matches
        .get_one::<PathBuf>("inventory_path")
        .expect("should be the first required argument");

    let changelog_path = matches
        .get_one::<PathBuf>("changelog_path")
        .expect("should be the second required argument");

    let platforms = matches
        .get_many::<SupportedNodeReleasePlatform>("platform")
        .expect("--platform should have a default value")
        .collect::<Vec<_>>();

    let format = matches
        .get_one::<OutputFormat>("format")
        .expect("--format should have a default value");

    eprintln!("Configuration:");
    eprintln!("  Inventory path: {}", inventory_path.display());
    eprintln!("  Changelog path: {}", changelog_path.display());
    eprintln!("  Platforms: {:?}", platforms);
    eprintln!("  Format: {:?}", format);

    eprintln!("Importing trusted release keys...");
    let node_release_keys = trusted_release_keys::import().await;

    eprintln!("Loading releases from inventory...");
    let inventory_artifacts = node_releases::from_inventory(inventory_path);

    eprintln!("Fetching upstream releases from nodejs.org...");
    let upstream_artifacts =
        node_releases::from_upstream(&inventory_artifacts, &platforms, node_release_keys).await;

    eprintln!("Writing inventory...");
    output::write_inventory(inventory_path, &upstream_artifacts);

    eprintln!("Writing changelog...");
    output::write_changelog(
        changelog_path,
        &upstream_artifacts,
        &inventory_artifacts,
        format,
    );
}

/// This is a subset of the platforms that are provided by nodejs.org which correspond to:
/// - `linux-x64` and `linux-arm64` in our CNBs
/// - `linux-x64` in our classic buildpacks
#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub(crate) enum SupportedNodeReleasePlatform {
    LinuxX64,   // becomes "linux-x64"
    LinuxArm64, // becomes "linux-arm64"
}

impl SupportedNodeReleasePlatform {
    pub(crate) fn release_file(&self) -> String {
        match self {
            SupportedNodeReleasePlatform::LinuxX64 => "linux-x64",
            SupportedNodeReleasePlatform::LinuxArm64 => "linux-arm64",
        }
        .to_string()
    }

    pub(crate) fn os(&self) -> Os {
        Os::Linux
    }

    pub(crate) fn arch(&self) -> Arch {
        match self {
            SupportedNodeReleasePlatform::LinuxX64 => Arch::Amd64,
            SupportedNodeReleasePlatform::LinuxArm64 => Arch::Arm64,
        }
    }
}

/// The format of the changelog in the classic buildpack *should* be in "Keep a Changelog" format,
/// but it's technically not. The CNB changelog does adhere to the strict version of the format.
#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub(crate) enum OutputFormat {
    Classic,        // becomes "classic"
    KeepAChangelog, // becomes "keep-a-changelog"
}
