use node_semver::{Range, Version as NSVersion};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct VersionError(String);
impl Error for VersionError {}
impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
            Err(e) => Err(VersionError(format!("{}", e))),
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
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "String")]
pub struct Requirement(Range);

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
            Err(error) => Err(VersionError(format!("{}", error))),
        }
    }

    #[must_use]
    pub fn any() -> Self {
        Requirement(Range::any())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handles_latest() {
        let result = Requirement::parse("latest");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("*", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_exact_versions() {
        let result = Requirement::parse("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_starts_with_v() {
        let result = Requirement::parse("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_semver_semantics() {
        let result = Requirement::parse(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_pipe_statements() {
        let result = Requirement::parse("^12 || ^13 || ^14");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(
                ">=12.0.0 <13.0.0-0||>=13.0.0 <14.0.0-0||>=14.0.0 <15.0.0-0",
                format!("{}", reqs)
            );
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals() {
        let result = Requirement::parse("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals_and_patch() {
        let result = Requirement::parse("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_v_within_string() {
        let result = Requirement::parse(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_v_with_space() {
        let result = Requirement::parse(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_equal_with_v() {
        let result = Requirement::parse("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_returns_error_for_invalid_reqs() {
        let result = Requirement::parse("12.%");
        println!("{:?}", result);

        assert!(result.is_err());
    }
}
