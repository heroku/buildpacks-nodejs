use crate::OutputFormat;
use crate::node_releases::NodeArtifact;
use keep_a_changelog_file::{ChangeGroup, Changelog};
use libherokubuildpack::inventory::Inventory;
use node_semver::Version;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub(super) fn write_inventory(inventory_path: &Path, upstream_artifacts: &[NodeArtifact]) {
    fs::write(
        inventory_path,
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
    .unwrap_or_else(|_| {
        panic!(
            "Failed to write inventory to '{}'",
            inventory_path.display()
        )
    });
}

pub(super) fn write_changelog(
    changelog_path: &Path,
    upstream_artifacts: &[NodeArtifact],
    inventory_artifacts: &[NodeArtifact],
    format: &OutputFormat,
) {
    let changes = [
        (
            ChangeGroup::Added,
            Vec::from_iter(
                upstream_artifacts
                    .iter()
                    .filter(|&artifact| !inventory_artifacts.contains(artifact)),
            ),
        ),
        (
            ChangeGroup::Removed,
            Vec::from_iter(
                inventory_artifacts
                    .iter()
                    .filter(|&artifact| !upstream_artifacts.contains(artifact)),
            ),
        ),
    ]
    .into_iter()
    .filter(|(_, artifact_diff)| !artifact_diff.is_empty())
    .collect::<Vec<_>>();

    match format {
        OutputFormat::Classic => write_classic_changelog_format(changelog_path, changes),
        OutputFormat::KeepAChangelog => write_keep_a_changelog_format(changelog_path, changes),
    }
}

fn write_keep_a_changelog_format(
    changelog_path: &Path,
    changes: Vec<(ChangeGroup, Vec<&NodeArtifact>)>,
) {
    let changelog_contents = fs::read_to_string(changelog_path)
        .unwrap_or_else(|_| panic!("Failed to read changelog at '{}'", changelog_path.display()));

    let mut changelog = Changelog::from_str(&changelog_contents)
        .unwrap_or_else(|_| panic!("Error parsing changelog at '{}'", changelog_path.display()));

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

    fs::write(changelog_path, changelog.to_string()).unwrap_or_else(|_| {
        panic!(
            "Failed to write to changelog to '{}'",
            changelog_path.display()
        )
    });
}

fn write_classic_changelog_format(
    changelog_path: &Path,
    changes: Vec<(ChangeGroup, Vec<&NodeArtifact>)>,
) {
    // utility enum to differentiate between before/after [Unreleased] section
    #[derive(PartialEq)]
    enum Section {
        Before,
        After,
    }

    let mut changelog = fs::read_to_string(changelog_path)
        .unwrap_or_else(|_| panic!("Failed to read changelog at '{}'", changelog_path.display()))
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

    fs::write(changelog_path, changelog.join("\n")).unwrap_or_else(|_| {
        panic!(
            "Failed to write to changelog at '{}'",
            changelog_path.display()
        )
    });
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
