use node_semver::{Range, Version};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct VerErr(String);
impl Error for VerErr {}
impl fmt::Display for VerErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct Ver(Version);
impl Ver {
    /// Parses a Node.js semver string as a `Ver`.
    ///
    /// # Errors
    ///
    /// Invalid Node.js semver strings will return a `VerErr`
    pub fn parse(version: &str) -> Result<Self, VerErr> {
        let trimmed = version.trim();
        match Version::parse(trimmed) {
            Ok(v) => Ok(Ver(v)),
            Err(e) => Err(VerErr(format!("{}", e))),
        }
    }
}
impl TryFrom<String> for Ver {
    type Error = VerErr;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Ver::parse(&val)
    }
}
impl fmt::Display for Ver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "String")]
pub struct Req(Range);

impl Req {
    /// Parses `package.json` version string into a Req. Handles
    /// these cases:
    ///
    /// * Any node-semver compatible string
    /// * "latest" as "*"
    /// * "~=" as "="
    ///
    /// # Errors
    ///
    /// Invalid version strings wil return a `VerErr`
    pub fn parse(requirement: &str) -> Result<Self, VerErr> {
        let trimmed = requirement.trim();
        if requirement == "latest" {
            return Ok(Req(Range::any()));
        }
        if trimmed.starts_with("~=") {
            let version = trimmed.replacen('=', "", 1);
            if let Ok(range) = Range::parse(version) {
                return Ok(Req(range));
            }
        }
        match Range::parse(&trimmed) {
            Ok(range) => Ok(Req(range)),
            Err(error) => Err(VerErr(format!("{}", error))),
        }
    }

    #[must_use]
    pub fn any() -> Self {
        Req(Range::any())
    }

    #[must_use]
    pub fn satisfies(&self, ver: &Ver) -> bool {
        self.0.satisfies(&ver.0)
    }
}

impl TryFrom<String> for Req {
    type Error = VerErr;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Req::parse(&val)
    }
}

impl fmt::Display for Req {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_handles_latest() {
        let result = Req::parse("latest");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("*", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_exact_versions() {
        let result = Req::parse("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_starts_with_v() {
        let result = Req::parse("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_semver_semantics() {
        let result = Req::parse(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_pipe_statements() {
        let result = Req::parse("^12 || ^13 || ^14");

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
        let result = Req::parse("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals_and_patch() {
        let result = Req::parse("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_v_within_string() {
        let result = Req::parse(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_v_with_space() {
        let result = Req::parse(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_equal_with_v() {
        let result = Req::parse("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_returns_error_for_invalid_reqs() {
        let result = Req::parse("12.%");
        println!("{:?}", result);

        assert!(result.is_err());
    }
}
