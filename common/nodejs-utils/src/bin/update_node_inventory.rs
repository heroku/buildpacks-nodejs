// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use anyhow::{Context, Result};
use heroku_inventory_utils::{
    checksum::Checksum,
    inv::{read_inventory_file, Arch, Artifact, Inventory, Os},
};
use keep_a_changelog_file::{ChangeGroup, Changelog};
use node_semver::Version;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::{collections::HashMap, env, fs};

const USAGE: &str = "Usage: update_inventory <path/to/inventory.toml> <path/to/CHANGELOG.md>";

/// Updates the local node.js inventory.toml with versions published on nodejs.org.
fn main() -> Result<()> {
    let inventory_path = env::args()
        .nth(1)
        .context(format!("Missing path to inventory file!\n\n{USAGE}"))?;

    let changelog_path = env::args()
        .nth(2)
        .context(format!("Missing path to changelog file!\n\n{USAGE}"))?;

    let inventory_artifacts: Vec<Artifact<Version, Sha256>> =
        read_inventory_file(&inventory_path)?.artifacts;

    let upstream_artifacts = fetch_upstream_artifacts(&inventory_artifacts)?;

    write_inventory(inventory_path, &upstream_artifacts)?;

    write_changelog(changelog_path, &upstream_artifacts, &inventory_artifacts)?;

    Ok(())
}

fn write_inventory(
    inventory_path: impl Into<PathBuf>,
    upstream_artifacts: &[Artifact<Version, Sha256>],
) -> Result<()> {
    toml::to_string(&Inventory {
        artifacts: {
            let mut artifacts = Vec::from_iter(upstream_artifacts.to_owned());
            artifacts.sort_by(|a, b| {
                if a.version == b.version {
                    b.arch.to_string().cmp(&a.arch.to_string())
                } else {
                    b.version.cmp(&a.version)
                }
            });
            artifacts
        },
    })
    .context("Error serializing inventory as toml")
    .and_then(|contents| {
        fs::write(inventory_path.into(), contents).context("Error writing inventory file")
    })
}

fn write_changelog(
    changelog_path: impl Into<PathBuf>,
    upstream_artifacts: &[Artifact<Version, Sha256>],
    inventory_artifacts: &[Artifact<Version, Sha256>],
) -> Result<()> {
    let changelog_path = changelog_path.into();

    let mut changelog = fs::read_to_string(&changelog_path)
        .context("Reading changelog")
        .and_then(|contents| {
            Changelog::from_str(&contents).context(format!(
                "Error parsing changelog at '{}'",
                changelog_path.display()
            ))
        })?;

    let changes = [
        (
            ChangeGroup::Added,
            Vec::from_iter(difference(upstream_artifacts, inventory_artifacts)),
        ),
        (
            ChangeGroup::Removed,
            Vec::from_iter(difference(inventory_artifacts, upstream_artifacts)),
        ),
    ];

    changes
        .into_iter()
        .filter(|(_, artifact_diff)| !artifact_diff.is_empty())
        .for_each(|(change_group, artifact_diff)| {
            println!("### {change_group}\n");

            let versions = {
                let mut os_arch_labels_by_version: HashMap<Version, BTreeSet<String>> =
                    HashMap::new();
                for artifact in artifact_diff {
                    os_arch_labels_by_version
                        .entry(artifact.version.clone())
                        .or_default()
                        .insert(format!("{}-{}", artifact.os, artifact.arch));
                }
                let mut sorted_versions = os_arch_labels_by_version.into_iter().collect::<Vec<_>>();
                sorted_versions.sort_by(|(version_a, _), (version_b, _)| version_b.cmp(version_a));
                sorted_versions
            };

            for (version, os_arch_labels) in versions {
                let os_arch_labels = os_arch_labels.into_iter().collect::<Vec<_>>().join(", ");
                println!("- Node.js {version} ({os_arch_labels})");
                changelog.unreleased.add(
                    change_group.clone(),
                    format!("{version} ({os_arch_labels})"),
                );
            }
            println!();
        });

    fs::write(changelog_path, changelog.to_string()).context("Failed to write to changelog")
}

/// Finds the difference between two slices.
fn difference<'a, T: Eq>(a: &'a [T], b: &'a [T]) -> Vec<&'a T> {
    a.iter().filter(|&artifact| !b.contains(artifact)).collect()
}

fn fetch_upstream_artifacts(
    inventory_artifacts: &[Artifact<Version, Sha256>],
) -> Result<Vec<Artifact<Version, Sha256>>> {
    let mut upstream_artifacts = vec![];
    for release in list_releases()? {
        if release.version >= Version::parse("0.8.6")? {
            let supported_platforms = [
                ("linux-arm64", Os::Linux, Arch::Arm64),
                ("linux-x64", Os::Linux, Arch::Amd64),
            ];

            for (file, os, arch) in supported_platforms {
                if !release.files.contains(&file.to_string()) {
                    continue;
                }

                if let Some(artifact) = inventory_artifacts
                    .iter()
                    .find(|x| x.arch == arch && x.os == os && x.version == release.version)
                {
                    upstream_artifacts.push(artifact.clone());
                } else {
                    let filename = format!("node-v{}-{}.tar.gz", release.version, file);

                    let shasums = fetch_checksums(&release.version)?;
                    let checksum_hex = shasums
                        .get(&filename)
                        .ok_or_else(|| anyhow::anyhow!("Checksum not found for {}", filename))?;

                    upstream_artifacts.push(Artifact::<Version, Sha256> {
                        url: format!(
                            "https://nodejs.org/download/release/v{}/{filename}",
                            release.version
                        ),
                        version: release.version.clone(),
                        checksum: Checksum::try_from(checksum_hex.to_owned())?,
                        arch,
                        os,
                    });
                }
            }
        }
    }
    Ok(upstream_artifacts)
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
            match (parts.next(), parts.next(), parts.next()) {
                (Some(checksum), Some(filename), None) => Some((
                    // Some of the checksum filenames contain a leading `./` (e.g.
                    // https://nodejs.org/download/release/v0.11.6/SHASUMS256.txt)
                    filename.trim_start_matches("./").to_string(),
                    checksum.to_string(),
                )),
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
