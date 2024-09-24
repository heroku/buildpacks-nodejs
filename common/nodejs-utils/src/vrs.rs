use crate::{distribution::Distribution, s3};
use anyhow::anyhow;
use libherokubuildpack::inventory::version::VersionRequirement;
use node_semver::{Range, Version as NSVersion};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{error::Error, fmt, str::FromStr};

#[derive(Debug)]
pub struct VersionError(String);
impl Error for VersionError {}
impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "String")]
pub struct Version(NSVersion);

impl Version {
    /// Parses a Node.js semver string as a `Version`.
    ///
    /// # Errors
    ///
    /// Invalid Node.js semver strings will return a `VersionError`
    pub fn parse(version: &str) -> Result<Self, VersionError> {
        let trimmed = version.trim();
        match NSVersion::parse(trimmed) {
            Ok(v) => Ok(Version(v)),
            Err(e) => Err(VersionError(format!("{e}"))),
        }
    }

    /// Returns the major version identifier.
    #[must_use]
    pub fn major(&self) -> u64 {
        self.0.major
    }

    /// Returns the minor version identifier.
    #[must_use]
    pub fn minor(&self) -> u64 {
        self.0.minor
    }

    /// Returns the patch version identifier.
    #[must_use]
    pub fn patch(&self) -> u64 {
        self.0.patch
    }
}

impl TryFrom<String> for Version {
    type Error = VersionError;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Version::parse(&val)
    }
}

impl FromStr for Version {
    type Err = VersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "String")]
pub struct Requirement(Range);

impl VersionRequirement<Version> for Requirement {
    fn satisfies(&self, version: &Version) -> bool {
        self.satisfies(version)
    }
}

impl Requirement {
    /// Parses `package.json` version string into a Requirement. Handles
    /// these cases:
    ///
    /// * Any node-semver compatible string
    /// * "latest" as "*"
    /// * "~=" as "="
    ///
    /// # Errors
    ///
    /// Invalid version strings wil return a `VersionErr`
    pub fn parse(requirement: &str) -> Result<Self, VersionError> {
        let trimmed = requirement.trim();
        if requirement == "latest" {
            return Ok(Requirement(Range::any()));
        }
        if trimmed.starts_with("~=") {
            let version = trimmed.replacen('=', "", 1);
            if let Ok(range) = Range::parse(version) {
                return Ok(Requirement(range));
            }
        }
        match Range::parse(trimmed) {
            Ok(range) => Ok(Requirement(range)),
            Err(error) => Err(VersionError(format!("{error}"))),
        }
    }

    #[must_use]
    pub fn satisfies(&self, ver: &Version) -> bool {
        self.0.satisfies(&ver.0)
    }
}

impl TryFrom<String> for Requirement {
    type Error = VersionError;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Requirement::parse(&val)
    }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) type VersionSet = HashSet<Version>;

impl TryFrom<s3::BucketContent> for VersionSet {
    type Error = anyhow::Error;

    /// # Failures
    /// These are the possible errors that can occur when calling this function:
    ///
    /// * Regex missing matching captures against `Content#key`
    /// * `Version::parse` fails to parse the version found in the `Content#key`
    fn try_from(content: s3::BucketContent) -> Result<Self, Self::Error> {
        let rex = content
            .prefix
            .parse::<Distribution>()?
            .mirrored_path_regex()?;

        content
            .contents
            .iter()
            .map(|content| {
                rex.captures(&content.key)
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn parse_handles_latest() {
        let result = Requirement::parse("latest");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("*", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_exact_versions() {
        let result = Requirement::parse("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_starts_with_v() {
        let result = Requirement::parse("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_semver_semantics() {
        let result = Requirement::parse(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_pipe_statements() {
        let result = Requirement::parse("^12 || ^13 || ^14");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(
                ">=12.0.0 <13.0.0-0||>=13.0.0 <14.0.0-0||>=14.0.0 <15.0.0-0",
                format!("{reqs}")
            );
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals() {
        let result = Requirement::parse("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals_and_patch() {
        let result = Requirement::parse("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_v_within_string() {
        let result = Requirement::parse(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_v_with_space() {
        let result = Requirement::parse(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_handles_equal_with_v() {
        let result = Requirement::parse("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{reqs}"));
        }
    }

    #[test]
    fn parse_returns_error_for_invalid_reqs() {
        let result = Requirement::parse("12.%");
        println!("{result:?}");

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
            ..Default::default()
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
}
