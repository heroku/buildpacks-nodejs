use fun_run::NamedOutput;
use libherokubuildpack::inventory::version::VersionRequirement;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, str::FromStr};

#[derive(Debug, PartialEq)]
pub struct VersionError(String);

impl Error for VersionError {}
impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "String")]
pub struct Version(node_semver::Version);

impl Version {
    /// Parses a Node.js semver string as a `Version`.
    ///
    /// # Errors
    ///
    /// Invalid Node.js semver strings will return a `VersionError`
    pub fn parse(version: &str) -> Result<Self, VersionError> {
        let trimmed = version.trim();
        match node_semver::Version::parse(trimmed) {
            Ok(v) => Ok(Version(v)),
            Err(e) => Err(VersionError(format!("{e}"))),
        }
    }

    /// Returns the major version identifier.
    #[must_use]
    pub fn major(&self) -> u64 {
        self.0.major
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

impl TryFrom<Result<NamedOutput, fun_run::CmdError>> for Version {
    type Error = VersionCommandError;

    fn try_from(val: Result<NamedOutput, fun_run::CmdError>) -> Result<Self, Self::Error> {
        val.map_err(VersionCommandError::Command)
            .and_then(|output| {
                let stdout = output.stdout_lossy();
                stdout
                    .parse::<Version>()
                    .map_err(|e| VersionCommandError::Parse(stdout, e))
            })
    }
}

#[derive(Debug)]
pub enum VersionCommandError {
    Command(fun_run::CmdError),
    Parse(String, VersionError),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "String")]
pub struct VersionRange {
    range: node_semver::Range,
    original: String,
}

impl VersionRequirement<Version> for VersionRange {
    fn satisfies(&self, version: &Version) -> bool {
        self.satisfies(version)
    }
}

impl VersionRange {
    /// Parses `package.json` version string into a `VersionRange`. Handles
    /// these cases:
    ///
    /// * Any node-semver compatible string
    /// * "latest" as "*"
    /// * "~=" as "="
    ///
    /// # Errors
    ///
    /// Invalid version strings wil return a `VersionError`
    pub fn parse(requirement: &str) -> Result<Self, VersionError> {
        let trimmed = requirement.trim();
        if requirement == "latest" {
            return Ok(VersionRange {
                range: node_semver::Range::any(),
                original: requirement.to_string(),
            });
        }
        if trimmed.starts_with("~=") {
            let version = trimmed.replacen('=', "", 1);
            if let Ok(range) = node_semver::Range::parse(version) {
                return Ok(VersionRange {
                    range,
                    original: requirement.to_string(),
                });
            }
        }
        match node_semver::Range::parse(trimmed) {
            Ok(range) => Ok(VersionRange {
                range,
                original: requirement.to_string(),
            }),
            Err(error) => Err(VersionError(format!("{error}"))),
        }
    }

    #[must_use]
    pub fn satisfies(&self, ver: &Version) -> bool {
        self.range.satisfies(&ver.0)
    }

    #[must_use]
    pub fn allows_any(&self, other: &VersionRange) -> bool {
        self.range.allows_any(&other.range)
    }
}

impl TryFrom<String> for VersionRange {
    type Error = VersionError;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        VersionRange::parse(&val)
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}

pub type NodejsArtifact =
    libherokubuildpack::inventory::artifact::Artifact<Version, sha2::Sha256, Option<()>>;
pub type NodejsInventory =
    libherokubuildpack::inventory::Inventory<Version, sha2::Sha256, Option<()>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handles_latest() {
        let result = VersionRange::parse("latest");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("*", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_exact_versions() {
        let result = VersionRange::parse("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_starts_with_v() {
        let result = VersionRange::parse("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_semver_semantics() {
        let result = VersionRange::parse(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_pipe_statements() {
        let result = VersionRange::parse("^12 || ^13 || ^14");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(
                ">=12.0.0 <13.0.0-0||>=13.0.0 <14.0.0-0||>=14.0.0 <15.0.0-0",
                format!("{}", reqs.range)
            );
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals() {
        let result = VersionRange::parse("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals_and_patch() {
        let result = VersionRange::parse("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_v_within_string() {
        let result = VersionRange::parse(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_v_with_space() {
        let result = VersionRange::parse(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_equal_with_v() {
        let result = VersionRange::parse("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_returns_error_for_invalid_reqs() {
        let result = VersionRange::parse("12.%");
        assert!(result.is_err());
    }
}
