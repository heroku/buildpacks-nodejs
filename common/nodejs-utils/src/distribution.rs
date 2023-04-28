use crate::{
    inv::{Inventory, Release, BUCKET, REGION},
    nodejs_org, s3,
    vrs::{Version,Requirement},
};
use anyhow::anyhow;
use regex::Regex;
use std::{collections::HashSet, str::FromStr, fmt};
use std::convert::TryFrom;

const NODE_MIRRORED_DISTRIBUTION_REGEX: &str =
    r"node/(?P<channel>\w+)/(?P<arch>[\w-]+)/node-v(?P<version>\d+\.\d+\.\d+)[\w-]+\.tar\.gz";
const YARN_MIRRORED_DISTRIBUTION_REGEX: &str =
    r"yarn/(?P<channel>\w+)/yarn-v(?P<version>\d+\.\d+\.\d+(-[\w\.]+)?)\.tar\.gz";

#[derive(Debug,Clone,Copy)]
pub enum Distribution {
    Yarn,
    Node,
}

impl FromStr for Distribution {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Node.js" | "node" => Ok(Self::Node),
            "Yarn" | "yarn" => Ok(Self::Yarn),
            other => Err(anyhow!("Unknown distribution: {other}")),
        }
    }
}

impl fmt::Display for Distribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node => write!(f, "Node.js"),
            Self::Yarn => write!(f, "Yarn"),
        }
    }
}

impl Distribution {
    /// List all versions of a distribution from it's upstream/source location.
    ///
    /// # Errors
    /// - http call failures
    /// - http response parsing errors
    pub fn upstream_versions(&self) -> anyhow::Result<VersionSet> {
        match self {
            Self::Node => list_upstream_node_versions(),
            Self::Yarn => list_upstream_yarn_versions(),
        }
    }

    /// List all versions of a distribution from it's mirrored location.
    ///
    /// # Errors
    /// - http call failures
    /// - http response parsing errors
    pub fn mirrored_versions(&self) -> anyhow::Result<VersionSet> {
        s3::list_objects(BUCKET, REGION, self.bucket_prefix())?.try_into()
    }

    /// Build an inventory from the releases listed in the mirrored location.
    ///
    /// # Errors
    /// - http call failures
    /// - http response parsing errors
    pub fn mirrored_inventory(&self) -> anyhow::Result<Inventory> {
        s3::list_objects(BUCKET, REGION, self.bucket_prefix())?.try_into()
    }

    /// Filter inactive distribution versions and return as a `VersionSet`.
    ///
    /// # Errors
    /// - Active semver range for the distribution is a valid version.
    pub fn filter_inactive_versions<'i, I>(&self, iter: I) -> anyhow::Result<VersionSet> where I: Iterator<Item=&'i Version> {
        let req = self.active_requirement()?;
        Ok(iter.filter(|v| req.satisfies(v)).cloned().collect())
    }

    fn bucket_prefix(&self) -> &str {
        match self {
            Self::Node => "node",
            Self::Yarn => "yarn",
        }
    }
    fn active_requirement(self) -> anyhow::Result<Requirement> {
        Requirement::parse(
            match self {
                Self::Node => ">= 16",
                Self::Yarn => ">= 1.22",
            }
        ).map_err(|e| anyhow!("{e}"))
    }
}

pub type VersionSet = HashSet<Version>;

fn list_upstream_node_versions() -> anyhow::Result<VersionSet> {
    nodejs_org::list_releases()?
        .iter()
        .map(|rls| {
            Version::parse(&rls.version).map_err(|e| {
                anyhow!("Couldn't parse upstream nodejs.org version as a version: {e}")
            })
        })
        .collect()
}

fn list_upstream_yarn_versions() -> anyhow::Result<VersionSet> {
    Version::parse("1.22.14").map(|v| VersionSet::from([v])).map_err(|_|anyhow!("Couldn't parse"))
}


impl TryFrom<s3::BucketContent> for VersionSet {
    type Error = anyhow::Error;

    /// # Failures
    /// These are the possible errors that can occur when calling this function:
    ///
    /// * Regex missing matching captures against `Content#key`
    /// * `Version::parse` fails to parse the version found in the `Content#key`
    fn try_from(result: s3::BucketContent) -> Result<Self, Self::Error> {
        let inv = &result.prefix;
        let vrs_rex = match inv.as_str() {
            "yarn" => Regex::new(YARN_MIRRORED_DISTRIBUTION_REGEX),
            "node" => Regex::new(NODE_MIRRORED_DISTRIBUTION_REGEX),
            i => Err(regex::Error::Syntax(format!(
                "Unknown S3 inventory prefix: {i}"
            ))),
        }?;

        result
            .contents
            .iter()
            .map(|content| {
                vrs_rex
                    .captures(&content.key)
                    .ok_or(anyhow!(
                        "Couldn't match the bucket content key to a known format: {}",
                        content.key
                    ))
                    .and_then(|capts| {
                        capts.name("version").ok_or(anyhow!(
                            "Couldn't find a version number in the bucket content key: {}",
                            content.key
                        ))
                    })
                    .and_then(|vrs_match| {
                        Version::parse(vrs_match.as_str()).map_err(|e| {
                            anyhow!("Couldn't serialize bucket content key as a Version: {e}")
                        })
                    })
            })
            .collect()
    }
}

impl TryFrom<s3::BucketContent> for Inventory {
    type Error = anyhow::Error;

    /// # Failures
    /// These are the possible errors that can occur when calling this function:
    ///
    /// * Regex missing matching captures against `Content#key`
    /// * `Version::parse` fails to parse the version found in the `Content#key`
    fn try_from(result: s3::BucketContent) -> Result<Self, Self::Error> {
        let inv = &result.prefix;
        let version_regex = match inv.as_str() {
            "yarn" => Regex::new(YARN_MIRRORED_DISTRIBUTION_REGEX),
            "node" => Regex::new(NODE_MIRRORED_DISTRIBUTION_REGEX),
            i => Err(regex::Error::Syntax(format!(
                "Unknown S3 inventory prefix: {i}"
            ))),
        }?;

        let releases: anyhow::Result<Vec<Release>> = result
            .contents
            .iter()
            .map(|content| {
                let capture = version_regex.captures(&content.key).ok_or_else(|| {
                    anyhow!("No valid version found in content: {}", &content.key)
                })?;
                let channel = capture.name("channel").ok_or_else(|| {
                    anyhow!("Could not find channel in content: {}", &content.key)
                })?;
                let version_number = capture.name("version").ok_or_else(|| {
                    anyhow!("Could not find version in content: {}", &content.key)
                })?;
                let arch = capture.name("arch");

                Ok(Release {
                    arch: arch.map(|a| a.as_str().to_string()),
                    version: Version::parse(version_number.as_str())?,
                    channel: channel.as_str().to_string(),
                    // Amazon S3 returns a quoted string for ETags
                    etag: Some(content.etag.replace('\"', "")),
                    url: format!(
                        "https://{BUCKET}.s3.{REGION}.amazonaws.com/{}",
                        &content.key
                    ),
                })
            })
            .collect();

        Ok(Self {
            name: inv.to_string(),
            releases: releases?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn it_converts_s3_result_to_inv() {
        let etag = "739c200ca266266ff150ad4d89b83205";
        let content = s3::Content {
            key: "node/release/darwin-x64/node-v0.10.0-darwin-x64.tar.gz".to_string(),
            ..Default::default()
        };
        let bucket_content = s3::BucketContent {
            prefix: "node".to_string(),
            contents: vec![content],
        };

        let result = Inventory::try_from(bucket_content);
        assert!(result.is_ok());
        if let Ok(inv) = result {
            assert_eq!(Some(String::from(etag)), inv.releases[0].etag);
        }
    }

    #[test]
    fn it_converts_s3_result_to_inv_arch_optional() {
        let content = s3::Content {
            key: "yarn/release/yarn-v0.16.0.tar.gz".to_string(),
            ..Default::default()
        };
        let bucket_content = s3::BucketContent {
            prefix: "yarn".to_string(),
            contents: vec![content],
        };

        let result = Inventory::try_from(bucket_content);
        assert!(result.is_ok());
    }

    #[test]
    fn it_fails_to_convert_s3_result_to_inv() {
        let content = s3::Content {
            key: "garbage".to_string(),
            last_modified: Utc::now(),
            etag: "\"e4cc76bea92fabb664edadc4db14a8f2\"".to_string(),
            size: 7_234_362,
            storage_class: "STANDARD".to_string(),
        };
        let bucket_content = s3::BucketContent {
            prefix: "yarn".to_string(),
            contents: vec![content],
        };

        let result = Inventory::try_from(bucket_content);
        assert!(result.is_err());
    }

    #[test]
    fn try_from_bucket_content_for_version_set_succeeds() {
        let content = s3::Content {
            key: "node/release/darwin-x64/node-v18.10.2-darwin-x64.tar.gz".to_string(),
            last_modified: Utc::now(),
            etag: "123abcdefg".to_string(),
            size: 4_065_868,
            storage_class: "STANDARD".to_string(),
        };
        let bucket_content = s3::BucketContent {
            prefix: "node".to_string(),
            contents: vec![content],
        };

        let vrs_set = VersionSet::try_from(bucket_content)
            .expect("Expected to convert bucket content to a version set");
        println!("vrs_set: {vrs_set:?}");

        let expected = Version::parse("18.10.2").expect("Expected to parse a valid version");
        let actual = vrs_set
            .get(&expected)
            .expect("Expected to find a matching version");
        assert_eq!(&expected, actual);
    }

    #[test]
    fn test_list_upstream_node_versions() {
        let expected_version = Version::parse("20.0.0").expect("Expected to parse a valid version");
        let versions = list_upstream_node_versions()
            .expect("Expected to list upstream remote versions, but got an error");
        let actual_version = versions
            .get(&expected_version)
            .expect("Expected to find a matching version");
        assert_eq!(&expected_version, actual_version);
    }
}
