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

    /// Returns the minor version identifier.
    #[must_use]
    pub fn minor(&self) -> u64 {
        self.0.minor
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

// --- Release schedule types ---

/// A newtype wrapper around `time::Date` that serializes as a native TOML date.
///
/// This is needed because `Release<R, E, M>` uses `E: Serialize` directly,
/// so we cannot use `#[serde(with = "toml_datetime_compat")]` on the field.
/// Instead, this wrapper delegates to `toml_datetime_compat` in its own
/// `Serialize`/`Deserialize` implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TomlDate(pub time::Date);

impl Serialize for TomlDate {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        toml_datetime_compat::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for TomlDate {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        toml_datetime_compat::deserialize(deserializer).map(TomlDate)
    }
}

impl fmt::Display for TomlDate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a Node.js release line like "v24" or "v0.12".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodejsReleaseLine {
    major: u64,
    minor: Option<u64>,
}

impl NodejsReleaseLine {
    /// Returns the major version number of this release line.
    #[must_use]
    pub fn major(&self) -> u64 {
        self.major
    }
}

impl Ord for NodejsReleaseLine {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.major, self.minor.unwrap_or(0)).cmp(&(other.major, other.minor.unwrap_or(0)))
    }
}

impl PartialOrd for NodejsReleaseLine {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for NodejsReleaseLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.minor {
            Some(minor) => write!(f, "v{}.{}", self.major, minor),
            None => write!(f, "v{}", self.major),
        }
    }
}

impl From<NodejsReleaseLine> for String {
    fn from(r: NodejsReleaseLine) -> String {
        r.to_string()
    }
}

impl TryFrom<String> for NodejsReleaseLine {
    type Error = VersionError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        let s = s.strip_prefix('v').ok_or_else(|| {
            VersionError(format!("invalid release line '{s}': must start with 'v'"))
        })?;
        let parts: Vec<&str> = s.splitn(2, '.').collect();
        match parts.as_slice() {
            [major] => {
                let major = major
                    .parse::<u64>()
                    .map_err(|e| VersionError(format!("invalid major version: {e}")))?;
                Ok(NodejsReleaseLine { major, minor: None })
            }
            [major, minor] => {
                let major = major
                    .parse::<u64>()
                    .map_err(|e| VersionError(format!("invalid major version: {e}")))?;
                let minor = minor
                    .parse::<u64>()
                    .map_err(|e| VersionError(format!("invalid minor version: {e}")))?;
                Ok(NodejsReleaseLine {
                    major,
                    minor: Some(minor),
                })
            }
            _ => Err(VersionError(format!(
                "invalid release line 'v{s}': expected 'vN' or 'vN.M'"
            ))),
        }
    }
}

impl Serialize for NodejsReleaseLine {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for NodejsReleaseLine {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        NodejsReleaseLine::try_from(s).map_err(serde::de::Error::custom)
    }
}

impl VersionRequirement<Version> for NodejsReleaseLine {
    fn satisfies(&self, version: &Version) -> bool {
        if self.major > 0 {
            // For major > 0, match any version with the same major
            version.major() == self.major
        } else {
            // For major == 0, we need a minor to match
            match self.minor {
                Some(minor) => {
                    if version.major() != 0 {
                        return false;
                    }
                    // Direct match on minor version
                    if version.minor() == minor {
                        return true;
                    }
                    // Special cases: development releases that don't have their
                    // own schedule entries. v0.10 also covers v0.9.x, and
                    // v0.12 also covers v0.11.x.
                    match minor {
                        10 => version.minor() == 9,
                        12 => version.minor() == 11,
                        _ => false,
                    }
                }
                None => version.major() == 0,
            }
        }
    }
}

/// Metadata for a Node.js release line, containing lifecycle dates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodejsReleaseMetadata {
    /// The date this release line was first published.
    #[serde(with = "toml_datetime_compat")]
    pub start: time::Date,
    /// The date this release line entered LTS status, if applicable.
    #[serde(with = "toml_datetime_compat", default)]
    pub lts: Option<time::Date>,
    /// The date this release line entered maintenance mode, if applicable.
    #[serde(with = "toml_datetime_compat", default)]
    pub maintenance: Option<time::Date>,
}

pub type NodejsRelease = libherokubuildpack::inventory::schedule::Release<
    NodejsReleaseLine,
    TomlDate,
    NodejsReleaseMetadata,
>;

pub type NodejsReleaseSchedule = libherokubuildpack::inventory::schedule::Schedule<
    NodejsReleaseLine,
    TomlDate,
    NodejsReleaseMetadata,
>;

/// Combined inventory and release schedule for Node.js.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodejsInventoryWithSchedule {
    /// The release schedule containing lifecycle information for each release line.
    #[serde(flatten)]
    pub schedule: NodejsReleaseSchedule,
    /// The artifact inventory containing downloadable Node.js versions.
    #[serde(flatten)]
    pub inventory: NodejsInventory,
}

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

    // --- NodejsReleaseLine tests ---

    #[test]
    fn release_line_parse_major_only() {
        let rl = NodejsReleaseLine::try_from("v24".to_string()).unwrap();
        assert_eq!(rl.major, 24);
        assert_eq!(rl.minor, None);
        assert_eq!(rl.to_string(), "v24");
    }

    #[test]
    fn release_line_parse_major_and_minor() {
        let rl = NodejsReleaseLine::try_from("v0.12".to_string()).unwrap();
        assert_eq!(rl.major, 0);
        assert_eq!(rl.minor, Some(12));
        assert_eq!(rl.to_string(), "v0.12");
    }

    #[test]
    fn release_line_parse_missing_v_prefix() {
        assert!(NodejsReleaseLine::try_from("24".to_string()).is_err());
    }

    #[test]
    fn release_line_parse_v_only() {
        assert!(NodejsReleaseLine::try_from("v".to_string()).is_err());
    }

    #[test]
    fn release_line_parse_empty() {
        assert!(NodejsReleaseLine::try_from(String::new()).is_err());
    }

    #[test]
    fn release_line_ordering() {
        let lines: Vec<NodejsReleaseLine> = ["v0.8", "v0.10", "v0.12", "v4", "v25", "v26"]
            .iter()
            .map(|s| NodejsReleaseLine::try_from(s.to_string()).unwrap())
            .collect();

        for window in lines.windows(2) {
            assert!(
                window[0] < window[1],
                "{} should be less than {}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn release_line_satisfies_major() {
        let v24 = NodejsReleaseLine::try_from("v24".to_string()).unwrap();
        assert!(v24.satisfies(&Version::parse("24.0.0").unwrap()));
        assert!(v24.satisfies(&Version::parse("24.5.1").unwrap()));
        assert!(!v24.satisfies(&Version::parse("25.0.0").unwrap()));
        assert!(!v24.satisfies(&Version::parse("23.0.0").unwrap()));
    }

    #[test]
    fn release_line_satisfies_zero_minor() {
        let v0_12 = NodejsReleaseLine::try_from("v0.12".to_string()).unwrap();
        assert!(v0_12.satisfies(&Version::parse("0.12.0").unwrap()));
        assert!(v0_12.satisfies(&Version::parse("0.12.18").unwrap()));
        assert!(!v0_12.satisfies(&Version::parse("0.10.0").unwrap()));
    }

    #[test]
    fn release_line_satisfies_development_release_edge_cases() {
        // v0.10 should also satisfy v0.9.x (development release)
        let v0_10 = NodejsReleaseLine::try_from("v0.10".to_string()).unwrap();
        assert!(v0_10.satisfies(&Version::parse("0.9.0").unwrap()));
        assert!(v0_10.satisfies(&Version::parse("0.10.0").unwrap()));

        // v0.12 should also satisfy v0.11.x (development release)
        let v0_12 = NodejsReleaseLine::try_from("v0.12".to_string()).unwrap();
        assert!(v0_12.satisfies(&Version::parse("0.11.0").unwrap()));
        assert!(v0_12.satisfies(&Version::parse("0.12.0").unwrap()));
    }

    #[test]
    fn inventory_with_schedule_round_trip() {
        let toml_str = r#"
[[releases]]
requirement = "v24"
end_of_life = 2028-04-30
metadata = { start = 2025-04-22, lts = 2025-10-28, maintenance = 2026-10-20 }

[[releases]]
requirement = "v25"
end_of_life = 2026-06-01
metadata = { start = 2025-04-22 }

[[artifacts]]
version = "25.0.0"
os = "linux"
arch = "amd64"
url = "https://nodejs.org/download/release/v25.0.0/node-v25.0.0-linux-x64.tar.gz"
checksum = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
"#;
        let parsed: NodejsInventoryWithSchedule =
            toml::from_str(toml_str).expect("should parse TOML");

        assert_eq!(parsed.schedule.releases.len(), 2);
        assert_eq!(parsed.schedule.releases[0].requirement.to_string(), "v24");
        assert_eq!(parsed.schedule.releases[1].requirement.to_string(), "v25");
        assert_eq!(parsed.inventory.artifacts.len(), 1);

        // Re-serialize and parse again to verify round-trip
        let reserialized = toml::to_string(&parsed).expect("should serialize");
        let reparsed: NodejsInventoryWithSchedule =
            toml::from_str(&reserialized).expect("should parse re-serialized TOML");
        assert_eq!(reparsed.schedule.releases.len(), 2);
        assert_eq!(reparsed.inventory.artifacts.len(), 1);
    }
}
