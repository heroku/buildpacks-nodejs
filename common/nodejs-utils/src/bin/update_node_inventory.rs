// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use anyhow::{Context, Result};
use heroku_inventory_utils::{
    checksum::Checksum,
    inv::{read_inventory_file, Arch, Artifact, Inventory, Os},
};
use node_semver::Version;
use serde::Deserialize;
use sha2::Sha256;
use std::{
    collections::{HashMap, HashSet},
    env, fs, process,
};

/// Updates the local node.js inventory.toml with versions published on nodejs.org.
fn main() {
    let inventory_path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: update_inventory <path/to/inventory.toml>");
        process::exit(2);
    });

    let inventory_artifacts: HashSet<Artifact<Version, Sha256>> =
        read_inventory_file(&inventory_path)
            .unwrap_or_else(|e| {
                eprintln!("Error reading inventory at '{inventory_path}': {e}");
                std::process::exit(1);
            })
            .artifacts
            .into_iter()
            .collect();

    let upstream_artifacts = list_upstream_artifacts().unwrap_or_else(|e| {
        eprintln!("Failed to fetch upstream node.js versions: {e}");
        process::exit(4);
    });

    let inventory = Inventory {
        artifacts: upstream_artifacts,
    };

    let toml = toml::to_string(&inventory).unwrap_or_else(|e| {
        eprintln!("Error serializing inventory as toml: {e}");
        process::exit(6);
    });

    fs::write(inventory_path, toml).unwrap_or_else(|e| {
        eprintln!("Error writing inventory file: {e}");
        process::exit(7);
    });

    let remote_artifacts: HashSet<Artifact<Version, Sha256>> =
        inventory.artifacts.into_iter().collect();

    [
        ("Added", &remote_artifacts - &inventory_artifacts),
        ("Removed", &inventory_artifacts - &remote_artifacts),
    ]
    .iter()
    .filter(|(_, artifact_diff)| !artifact_diff.is_empty())
    .for_each(|(action, artifacts)| {
        let mut list: Vec<&Artifact<Version, Sha256>> = artifacts.iter().collect();
        list.sort_by_key(|a| &a.version);
        println!(
            "{} {}.",
            action,
            list.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );
    });
}

fn list_upstream_artifacts() -> Result<Vec<Artifact<Version, Sha256>>, anyhow::Error> {
    let earliest_version =
        Version::parse("0.8.6").context("Failed to parse earliest Node.js version")?;

    list_releases()?
        .into_iter()
        .filter(|release| release.version >= earliest_version)
        .map(|release| get_release_artifacts(&release))
        .collect::<Result<Vec<_>>>()
        .map(|nested| nested.into_iter().flatten().collect())
}

fn get_release_artifacts(release: &NodeJSRelease) -> Result<Vec<Artifact<Version, Sha256>>> {
    let supported_platforms = HashMap::from([
        ("linux-arm64", (Os::Linux, Arch::Arm64)),
        ("linux-x64", (Os::Linux, Arch::Amd64)),
    ]);

    let shasums = fetch_checksums(&release.version)?;
    release
        .files
        .iter()
        .filter(|file| supported_platforms.contains_key(&file.as_str()))
        .map(|file| {
            let (os, arch) = supported_platforms
                .get(file.as_str())
                .ok_or_else(|| anyhow::anyhow!("Unsupported platform: {}", file))?;

            let filename = format!("node-v{}-{}.tar.gz", release.version, file);
            let checksum_hex = shasums
                .get(&filename)
                .ok_or_else(|| anyhow::anyhow!("Checksum not found for {}", filename))?;

            Ok(Artifact::<Version, Sha256> {
                url: format!(
                    "https://nodejs.org/download/release/v{}/{filename}",
                    release.version
                ),
                version: release.version.clone(),
                checksum: Checksum::try_from(checksum_hex.to_owned())?,
                arch: *arch,
                os: *os,
            })
        })
        .collect()
}

fn fetch_checksums(version: &Version) -> Result<HashMap<String, String>> {
    ureq::get(&format!(
        "https://nodejs.org/download/release/v{version}/SHASUMS256.txt"
    ))
    .call()?
    .into_string()
    .map_err(anyhow::Error::from)
    .map(|x| parse_shasums(&x))
}

// Parses a SHASUMS256.txt file into a map of filename to checksum.
// Lines are expected to be of the form `<checksum> <filename>`.
fn parse_shasums(input: &str) -> HashMap<String, String> {
    input
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next()) {
                (Some(checksum), Some(filename)) if parts.next().is_none() => {
                    Some((
                        // Some of the checksum filenames contain a leading `./` (e.g.
                        // https://nodejs.org/download/release/v0.11.6/SHASUMS256.txt)
                        filename.trim_start_matches("./").to_string(),
                        checksum.to_string(),
                    ))
                }
                _ => None,
            }
        })
        .collect()
}

const NODE_UPSTREAM_LIST_URL: &str = "https://nodejs.org/download/release/index.json";

#[derive(Deserialize, Debug)]
struct NodeJSRelease {
    pub(crate) version: Version,
    pub(crate) files: Vec<String>,
}

fn list_releases() -> Result<Vec<NodeJSRelease>> {
    ureq::get(NODE_UPSTREAM_LIST_URL)
        .call()
        .context("Failed to fetch nodejs.org release list")?
        .into_json::<Vec<NodeJSRelease>>()
        .context("Failed to parse nodejs.org release list from JSON")
}