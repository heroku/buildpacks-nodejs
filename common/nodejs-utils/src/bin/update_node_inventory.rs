// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use anyhow::{Context, Result};
use clap::{arg, ArgAction, Command};
use keep_a_changelog_file::{ChangeGroup, Changelog};
use libherokubuildpack::inventory::artifact::{Arch, Artifact, Os};
use libherokubuildpack::inventory::checksum::Checksum;
use libherokubuildpack::inventory::Inventory;
use node_semver::Version;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::{collections::HashMap, fs};

type NodeArtifact = Artifact<Version, Sha256, Option<()>>;

const PLATFORM_LINUX_X64: &str = "linux-x64";
const PLATFORM_LINUX_ARM64: &str = "linux-arm64";

/// Updates the local node.js inventory.toml with versions published on nodejs.org.
fn main() -> Result<()> {
    let matches = Command::new("nodejs-update-inventory")
        .arg(arg!(<inventory_path>))
        .arg(arg!(<changelog_path>))
        .arg(
            arg!(--platform <platform>)
                .action(ArgAction::Append)
                .value_parser(["linux-arm64", PLATFORM_LINUX_X64])
                .default_values(["linux-arm64", PLATFORM_LINUX_X64]),
        )
        .arg(
            arg!(--format <format>)
                .value_parser(["classic", "default"])
                .default_value("default"),
        )
        .get_matches();

    let inventory_path = matches
        .get_one::<String>("inventory_path")
        .expect("required argument")
        .to_string();

    let changelog_path = matches
        .get_one::<String>("changelog_path")
        .expect("required argument")
        .to_string();

    let platforms = matches
        .get_many::<String>("platform")
        .expect("has a default value")
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    let format = matches
        .get_one::<String>("format")
        .expect("has a default value")
        .to_string();

    let inventory_artifacts = fs::read_to_string(&inventory_path)?
        .parse::<Inventory<Version, Sha256, Option<()>>>()
        .unwrap_or(Inventory::default())
        .artifacts;

    let upstream_artifacts = fetch_upstream_artifacts(&inventory_artifacts, &platforms)?;

    write_inventory(inventory_path, &upstream_artifacts)?;

    write_changelog(
        changelog_path,
        &upstream_artifacts,
        &inventory_artifacts,
        &format,
    )?;

    Ok(())
}

fn write_inventory(
    inventory_path: impl Into<PathBuf>,
    upstream_artifacts: &[NodeArtifact],
) -> Result<()> {
    fs::write(
        inventory_path.into(),
        Inventory {
            artifacts: {
                let mut artifacts = upstream_artifacts.to_vec();
                artifacts.sort_by(|a, b| {
                    if a.version == b.version {
                        b.arch.to_string().cmp(&a.arch.to_string())
                    } else {
                        b.version.cmp(&a.version)
                    }
                });
                artifacts
            },
        }
        .to_string(),
    )
    .context("Error writing inventory file")
}

fn write_changelog(
    changelog_path: impl Into<PathBuf>,
    upstream_artifacts: &[NodeArtifact],
    inventory_artifacts: &[NodeArtifact],
    format: &str,
) -> Result<()> {
    let changelog_path = changelog_path.into();

    let changes = [
        (
            ChangeGroup::Added,
            Vec::from_iter(difference(upstream_artifacts, inventory_artifacts)),
        ),
        (
            ChangeGroup::Removed,
            Vec::from_iter(difference(inventory_artifacts, upstream_artifacts)),
        ),
    ]
    .into_iter()
    .filter(|(_, artifact_diff)| !artifact_diff.is_empty())
    .collect::<Vec<_>>();

    if format == "default" {
        write_default_changelog_format(changelog_path, changes)
    } else {
        write_classic_changelog_format(changelog_path, changes)
    }
}

fn write_default_changelog_format(
    changelog_path: PathBuf,
    changes: Vec<(ChangeGroup, Vec<&NodeArtifact>)>,
) -> Result<()> {
    let mut changelog = fs::read_to_string(&changelog_path)
        .context("Reading changelog")
        .and_then(|contents| {
            Changelog::from_str(&contents).context(format!(
                "Error parsing changelog at '{}'",
                changelog_path.display()
            ))
        })?;

    for (change_group, artifact_diff) in changes {
        println!("### {change_group}\n");
        for (version, os_arch_labels) in
            group_artifacts_into_archs_by_version_sorted(&artifact_diff)
        {
            let os_arch_labels = os_arch_labels.into_iter().collect::<Vec<_>>().join(", ");
            println!("- Node.js {version} ({os_arch_labels})");
            changelog.unreleased.add(
                change_group.clone(),
                format!("{version} ({os_arch_labels})"),
            );
        }
        println!();
    }

    fs::write(changelog_path, changelog.to_string()).context("Failed to write to changelog")
}

fn write_classic_changelog_format(
    changelog_path: PathBuf,
    changes: Vec<(ChangeGroup, Vec<&NodeArtifact>)>,
) -> Result<()> {
    // utility enum to differentiate between before/after [Unreleased] section
    #[derive(PartialEq)]
    enum Section {
        Before,
        After,
    }

    let mut changelog = fs::read_to_string(&changelog_path)
        .context("Reading changelog")?
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    // Figure out:
    // - where the unreleased section starts
    // - how many blank lines appear before the unreleased content
    // - how many lines of unreleased content are present
    // - how many blank lines appear after the unreleased content
    let mut lines = changelog.iter();
    let mut unreleased_section_start = 0;
    let mut new_lines_before_content = 0;
    let mut lines_of_content = 0;
    let mut new_lines_after_content = 0;
    let mut current_section = Section::Before;
    while let Some(line) = lines.next() {
        // read up until we hit the unreleased header
        if line.to_lowercase().contains("## [unreleased]") {
            // keep iterating from here to determine how many blank lines
            // are before/after the next section
            for line in lines.by_ref() {
                let line = line.trim();
                if line.is_empty() {
                    match current_section {
                        Section::Before => new_lines_before_content += 1,
                        Section::After => new_lines_after_content += 1,
                    }
                } else if line.starts_with('-') || line.starts_with('*') {
                    current_section = Section::After;
                    lines_of_content += 1;
                } else {
                    break;
                }
            }
            break;
        }
        unreleased_section_start += 1;
    }

    // remove leading blank lines
    while new_lines_before_content != 0 {
        changelog.remove(unreleased_section_start + 1);
        new_lines_before_content -= 1;
    }

    // remove trailing blank lines
    while new_lines_after_content != 0 {
        changelog.remove(unreleased_section_start + lines_of_content + 1);
        new_lines_after_content -= 1;
    }

    // now start edits right after the unreleased header
    let mut index = unreleased_section_start + 1;

    // add a single leading blank line
    changelog.insert(index, String::new());
    index += 1;

    for (change_group, artifact_diff) in changes {
        println!("### {change_group}\n");
        for (version, os_arch_labels) in
            group_artifacts_into_archs_by_version_sorted(&artifact_diff)
        {
            let os_arch_labels = os_arch_labels.into_iter().collect::<Vec<_>>().join(", ");
            println!("- Node.js {version} ({os_arch_labels})");
            let change = format!("- {change_group} Node.js {version} ({os_arch_labels})");
            changelog.insert(index, change);
            index += 1;
        }
        println!();
    }

    // insert a single trailing blank line
    changelog.insert(index + lines_of_content, String::new());

    fs::write(changelog_path, changelog.join("\n")).context("Failed to write to changelog")
}

fn group_artifacts_into_archs_by_version_sorted(
    artifact_diff: &[&NodeArtifact],
) -> Vec<(Version, BTreeSet<String>)> {
    let mut os_arch_labels_by_version: HashMap<Version, BTreeSet<String>> = HashMap::new();
    for artifact in artifact_diff {
        os_arch_labels_by_version
            .entry(artifact.version.clone())
            .or_default()
            .insert(format!("{}-{}", artifact.os, artifact.arch));
    }
    let mut sorted_versions = os_arch_labels_by_version.into_iter().collect::<Vec<_>>();
    sorted_versions.sort_by(|(version_a, _), (version_b, _)| version_b.cmp(version_a));
    sorted_versions
}

/// Finds the difference between two slices.
fn difference<'a, T: Eq>(a: &'a [T], b: &'a [T]) -> Vec<&'a T> {
    a.iter().filter(|&artifact| !b.contains(artifact)).collect()
}

fn fetch_upstream_artifacts(
    inventory_artifacts: &[Artifact<Version, Sha256, Option<()>>],
    platforms: &[String],
) -> Result<Vec<Artifact<Version, Sha256, Option<()>>>> {
    let mut upstream_artifacts = vec![];

    let mut supported_platforms = vec![];
    let linux_x64 = PLATFORM_LINUX_X64.to_string();
    if platforms.contains(&linux_x64) {
        supported_platforms.push((linux_x64, Os::Linux, Arch::Amd64));
    }
    let linux_arm64 = PLATFORM_LINUX_ARM64.to_string();
    if platforms.contains(&linux_arm64) {
        supported_platforms.push((linux_arm64, Os::Linux, Arch::Arm64));
    }

    for release in list_releases()? {
        if release.version >= Version::parse("0.8.6")? {
            for (file, os, arch) in &supported_platforms {
                if !release.files.contains(&file.to_string()) {
                    continue;
                }

                if let Some(artifact) = inventory_artifacts
                    .iter()
                    .find(|x| x.arch == *arch && x.os == *os && x.version == release.version)
                {
                    upstream_artifacts.push(artifact.clone());
                } else {
                    let filename = format!("node-v{}-{}.tar.gz", release.version, file);

                    let shasums = fetch_checksums(&release.version)?;
                    let checksum_hex = shasums
                        .get(&filename)
                        .ok_or_else(|| anyhow::anyhow!("Checksum not found for {}", filename))?;

                    upstream_artifacts.push(Artifact::<Version, Sha256, Option<()>> {
                        url: format!(
                            "https://nodejs.org/download/release/v{}/{filename}",
                            release.version
                        ),
                        version: release.version.clone(),
                        checksum: format!("sha256:{checksum_hex}").parse::<Checksum<Sha256>>()?,
                        arch: *arch,
                        os: *os,
                        metadata: None,
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
    .body_mut()
    .read_to_string()
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
        .body_mut()
        .read_json::<Vec<NodeJSRelease>>()
        .context("Failed to parse nodejs.org release list from JSON")
}
