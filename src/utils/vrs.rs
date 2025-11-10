use fun_run::NamedOutput;
use libherokubuildpack::inventory::version::VersionRequirement;
use node_semver::{Range, Version as NSVersion};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, str::FromStr};

#[derive(Debug, PartialEq)]
pub(crate) struct VersionError(String);
impl Error for VersionError {}
impl fmt::Display for VersionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(try_from = "String")]
pub(crate) struct Version(NSVersion);

impl Version {
    /// Parses a Node.js semver string as a `Version`.
    ///
    /// # Errors
    ///
    /// Invalid Node.js semver strings will return a `VersionError`
    pub(crate) fn parse(version: &str) -> Result<Self, VersionError> {
        let trimmed = version.trim();
        match NSVersion::parse(trimmed) {
            Ok(v) => Ok(Version(v)),
            Err(e) => Err(VersionError(format!("{e}"))),
        }
    }

    /// Returns the major version identifier.
    #[must_use]
    pub(crate) fn major(&self) -> u64 {
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
pub(crate) enum VersionCommandError {
    Command(fun_run::CmdError),
    Parse(String, VersionError),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "String")]
pub(crate) struct Requirement {
    range: Range,
    original: String,
}

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
    pub(crate) fn parse(requirement: &str) -> Result<Self, VersionError> {
        let trimmed = requirement.trim();
        if requirement == "latest" {
            return Ok(Requirement {
                range: Range::any(),
                original: requirement.to_string(),
            });
        }
        if trimmed.starts_with("~=") {
            let version = trimmed.replacen('=', "", 1);
            if let Ok(range) = Range::parse(version) {
                return Ok(Requirement {
                    range,
                    original: requirement.to_string(),
                });
            }
        }
        match Range::parse(trimmed) {
            Ok(range) => Ok(Requirement {
                range,
                original: requirement.to_string(),
            }),
            Err(error) => Err(VersionError(format!("{error}"))),
        }
    }

    #[must_use]
    pub(crate) fn satisfies(&self, ver: &Version) -> bool {
        self.range.satisfies(&ver.0)
    }

    #[must_use]
    pub(crate) fn allows_any(&self, other: &Requirement) -> bool {
        self.range.allows_any(&other.range)
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
        write!(f, "{}", self.original)
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
            assert_eq!("*", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_exact_versions() {
        let result = Requirement::parse("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_starts_with_v() {
        let result = Requirement::parse("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_semver_semantics() {
        let result = Requirement::parse(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_pipe_statements() {
        let result = Requirement::parse("^12 || ^13 || ^14");

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
        let result = Requirement::parse("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals_and_patch() {
        let result = Requirement::parse("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_v_within_string() {
        let result = Requirement::parse(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_v_with_space() {
        let result = Requirement::parse(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_equal_with_v() {
        let result = Requirement::parse("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_returns_error_for_invalid_reqs() {
        let result = Requirement::parse("12.%");
        assert!(result.is_err());
    }
}
