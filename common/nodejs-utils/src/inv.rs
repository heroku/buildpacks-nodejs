use crate::{
    distribution::Distribution,
    s3,
    vrs::{Requirement, Version},
};
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::fs;
use thiserror::Error;

/// Default/assumed operating system for node release lookups
#[cfg(target_os = "macos")]
const OS: &str = "darwin";
#[cfg(target_os = "linux")]
const OS: &str = "linux";

/// Default/assumed architecture for node release lookups
const ARCH: &str = "x64";

/// Represents a software inventory with releases.
#[derive(Debug, Deserialize, Serialize)]
pub struct Inventory {
    pub name: String,
    #[serde(default)]
    pub releases: Vec<Release>,
}

impl Inventory {
    /// Reads a software inventory toml file from a path into a `Inventory`.
    ///
    /// # Errors
    ///
    /// * File access error
    /// * Toml parsing error
    /// * Deserialization error
    pub fn read(path: &str) -> Result<Self, InventoryReadError> {
        let data = fs::read_to_string(path).map_err(InventoryReadError::Access)?;
        toml::from_str(&data).map_err(InventoryReadError::Parse)
    }

    /// Resolves the `Release` based on `semver-node::Range`.
    /// If no Release can be found, then `None` is returned.
    #[must_use]
    pub fn resolve(&self, req: &Requirement) -> Option<&Release> {
        let platform = format!("{OS}-{ARCH}");
        self.resolve_other(req, &platform, "release")
    }

    #[must_use]
    fn resolve_other(
        &self,
        version_requirements: &Requirement,
        platform: &str,
        channel: &str,
    ) -> Option<&Release> {
        let mut filtered_versions: Vec<&Release> = self
            .releases
            .iter()
            .filter(|version| {
                version.arch.as_deref().unwrap_or(platform) == platform
                    && version.channel == channel
            })
            .collect();
        // reverse sort, so latest is at the top
        filtered_versions.sort_by(|a, b| b.version.cmp(&a.version));

        filtered_versions
            .into_iter()
            .find(|rel| version_requirements.satisfies(&rel.version))
    }
}

#[derive(Error, Debug)]
pub enum InventoryReadError {
    #[error("Could not access inventory: {0}")]
    Access(std::io::Error),
    #[error("Could not parse inventory: {0}")]
    Parse(toml::de::Error),
}

/// Represents a inv release.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Release {
    pub version: Version,
    pub channel: String,
    pub arch: Option<String>,
    pub url: String,
    pub etag: Option<String>,
}

impl TryFrom<s3::BucketContent> for Inventory {
    type Error = anyhow::Error;

    /// # Failures
    /// These are the possible errors that can occur when calling this function:
    ///
    /// * Regex missing matching captures against `Content#key`
    /// * `Version::parse` fails to parse the version found in the `Content#key`
    fn try_from(bucket_content: s3::BucketContent) -> Result<Self, Self::Error> {
        let dist: Distribution = bucket_content.prefix.parse()?;
        let rex = dist.mirrored_path_regex()?;

        let releases: anyhow::Result<Vec<Release>> = bucket_content
            .contents
            .iter()
            .map(|content| {
                let capture = rex.captures(&content.key).ok_or_else(|| {
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
                        "https://{}.s3.{}.amazonaws.com/{}",
                        &bucket_content.name, &bucket_content.region, &content.key
                    ),
                })
            })
            .collect();

        Ok(Self {
            name: dist.to_string(),
            releases: releases?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distribution::{DEFAULT_BUCKET, DEFAULT_REGION};
    use chrono::Utc;

    fn url(version: &str, arch: &str, channel: &str) -> String {
        format!(
            "https://{DEFAULT_BUCKET}.s3.{DEFAULT_REGION}.amazonaws.com/node/{channel}/{arch}/node-v{version}-{arch}.tar.gz"
        )
    }

    fn release(ver: &Version, arch: &str, channel: &str) -> Release {
        Release {
            version: ver.clone(),
            channel: channel.to_string(),
            arch: Some(arch.to_string()),
            url: url(&ver.to_string(), arch, channel),
            etag: Some("a586044d93acb053d28dd6c0ddf95362".to_string()),
        }
    }

    fn create_inventory() -> Inventory {
        let releases = [
            "13.10.0", "13.10.1", "13.11.0", "13.12.0", "13.13.0", "13.14.0", "14.0.0", "15.0.0",
        ]
        .iter()
        .fold(vec![], |mut rels, ver| {
            let version = Version::parse(ver).unwrap();
            rels.push(release(&version, "darwin-x64", "release"));
            rels.push(release(&version, "linux-x64", "release"));
            rels
        });

        Inventory {
            name: "node".to_string(),
            releases,
        }
    }

    #[test]
    fn resolve_other_resolves_based_on_arch_and_channel() {
        let inv = create_inventory();
        let version_req = Requirement::parse("*").unwrap();

        let option = inv.resolve_other(&version_req, "linux-x64", "release");
        assert!(option.is_some());
        if let Some(release) = option {
            assert_eq!("15.0.0", format!("{}", release.version));
            assert_eq!("linux-x64", release.arch.as_ref().unwrap());
            assert_eq!("release", release.channel);
        }
    }

    #[test]
    fn resolve_other_handles_x_in_version_requirement() {
        let inv = create_inventory();
        let version_req = Requirement::parse("13.10.x").unwrap();

        let option = inv.resolve_other(&version_req, "linux-x64", "release");
        assert!(option.is_some());
        if let Some(release) = option {
            assert_eq!("13.10.1", format!("{}", release.version));
            assert_eq!("linux-x64", release.arch.as_ref().unwrap());
            assert_eq!("release", release.channel);
        }
    }

    #[test]
    fn resolve_returns_none_if_no_valid_version() {
        let inv = create_inventory();
        let version_req = Requirement::parse("18.0.0").unwrap();

        let option = inv.resolve(&version_req);
        assert!(option.is_none());
    }

    #[test]
    fn resolve_handles_semver_from_apps() {
        let releases = [
            "10.0.0", "10.1.0", "10.10.0", "10.11.0", "10.12.0", "10.13.0", "10.14.0", "10.14.1",
            "10.14.2", "10.15.0", "10.15.1", "10.15.2", "10.15.3", "10.2.0", "10.2.1", "10.3.0",
            "10.4.0", "10.4.1", "10.5.0", "10.6.0", "10.7.0", "10.8.0", "10.9.0", "11.0.0",
            "11.1.0", "11.10.0", "11.10.1", "11.11.0", "11.12.0", "11.13.0", "11.14.0", "11.2.0",
            "11.3.0", "11.4.0", "11.5.0", "11.6.0", "11.7.0", "11.8.0", "11.9.0", "6.0.0", "6.1.0",
            "6.10.0", "6.10.1", "6.10.2", "6.10.3", "6.11.0", "6.11.1", "6.11.2", "6.11.3",
            "6.11.4", "6.11.5", "6.12.0", "6.12.1", "6.12.2", "6.12.3", "6.13.0", "6.13.1",
            "6.14.0", "6.14.1", "6.14.2", "6.14.3", "6.14.4", "6.15.0", "6.15.1", "6.16.0",
            "6.17.0", "6.17.1", "6.2.0", "6.2.1", "6.2.2", "6.3.0", "6.3.1", "6.4.0", "6.5.0",
            "6.6.0", "6.7.0", "6.8.0", "6.8.1", "6.9.0", "6.9.1", "6.9.2", "6.9.3", "6.9.4",
            "6.9.5", "8.0.0", "8.1.0", "8.1.1", "8.1.2", "8.1.3", "8.1.4", "8.10.0", "8.11.0",
            "8.11.1", "8.11.2", "8.11.3", "8.11.4", "8.12.0", "8.13.0", "8.14.0", "8.14.1",
            "8.15.0", "8.15.1", "8.16.0", "8.2.0", "8.2.1", "8.3.0", "8.4.0", "8.5.0", "8.6.0",
            "8.7.0", "8.8.0", "8.8.1", "8.9.0", "8.9.1", "8.9.2", "8.9.3", "8.9.4",
        ]
        .map(|vers| Version::parse(vers).unwrap())
        .iter()
        .fold(vec![], |mut releases, version| {
            releases.push(release(version, "linux-x64", "release"));
            releases.push(release(version, "darwin-x64", "release"));
            releases
        });

        let inv = Inventory {
            name: "node".to_string(),
            releases,
        };

        for (input, version) in [
            ("10.x", "10.15.3"),
            ("10.*", "10.15.3"),
            ("10", "10.15.3"),
            ("8.x", "8.16.0"),
            ("^8.11.3", "8.16.0"),
            ("~8.11.3", "8.11.4"),
            (">= 6.0.0", "11.14.0"),
            ("^6.9.0 || ^8.9.0 || ^10.13.0", "10.15.3"),
            ("6.* || 8.* || >= 10.*", "11.14.0"),
            (">= 6.11.1 <= 10", "8.16.0"),
            (">=8.10 <11", "10.15.3"),
        ] {
            let version_req = Requirement::parse(input).unwrap();
            let option = inv.resolve(&version_req);

            println!("vr: {version_req}");
            assert!(option.is_some());

            println!("rv: {:?}", option.unwrap());
            if let Some(release) = option {
                assert_eq!(version, &format!("{}", release.version));
                assert_eq!("release", release.channel);
            }
        }
    }

    #[test]
    fn converts_s3_result_to_inv() {
        let etag = "739c200ca266266ff150ad4d89b83205";
        let content = s3::Content {
            key: "node/release/darwin-x64/node-v0.10.0-darwin-x64.tar.gz".to_string(),
            etag: etag.to_string(),
            ..Default::default()
        };
        let bucket_content = s3::BucketContent {
            prefix: "node".to_string(),
            contents: vec![content],
            ..Default::default()
        };

        let result = Inventory::try_from(bucket_content);
        assert!(result.is_ok());
        if let Ok(inv) = result {
            assert_eq!(Some(String::from(etag)), inv.releases[0].etag);
        }
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
            ..Default::default()
        };

        let result = Inventory::try_from(bucket_content);
        assert!(result.is_err());
    }
}
