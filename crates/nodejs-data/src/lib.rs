use fun_run::NamedOutput;
use libherokubuildpack::inventory::version::VersionRequirement;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt, str::FromStr};
use time::{Date, OffsetDateTime};

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

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(try_from = "String", into = "String")]
pub struct Range {
    range: node_semver::Range,
    original: String,
}

impl VersionRequirement<Version> for Range {
    fn satisfies(&self, version: &Version) -> bool {
        self.satisfies(version)
    }
}

impl Range {
    /// Parses `package.json` version string into a Range. Handles
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
            return Ok(Range {
                range: node_semver::Range::any(),
                original: requirement.to_string(),
            });
        }
        if trimmed.starts_with("~=") {
            let version = trimmed.replacen('=', "", 1);
            if let Ok(range) = node_semver::Range::parse(version) {
                return Ok(Range {
                    range,
                    original: requirement.to_string(),
                });
            }
        }
        match node_semver::Range::parse(trimmed) {
            Ok(range) => Ok(Range {
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
    pub fn allows_any(&self, other: &Range) -> bool {
        self.range.allows_any(&other.range)
    }
}

impl TryFrom<String> for Range {
    type Error = VersionError;
    fn try_from(val: String) -> Result<Self, Self::Error> {
        Range::parse(&val)
    }
}

impl From<Range> for String {
    fn from(range: Range) -> Self {
        range.original
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.original)
    }
}

pub type NodejsArtifact =
    libherokubuildpack::inventory::artifact::Artifact<Version, sha2::Sha256, Option<()>>;
pub type NodejsInventory =
    libherokubuildpack::inventory::Inventory<Version, sha2::Sha256, Option<()>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodejsReleaseMetadata {
    pub start: Date,
    #[serde(default)]
    pub lts: Option<Date>,
    #[serde(default)]
    pub maintenance: Option<Date>,
}

pub type NodejsReleaseSchedule = release_schedule::Schedule<Range, Date, NodejsReleaseMetadata>;
pub type NodejsRelease = release_schedule::Release<Range, Date, NodejsReleaseMetadata>;

#[must_use]
pub fn is_eol(release: &NodejsRelease) -> bool {
    OffsetDateTime::now_utc().date() >= release.end_of_life
}

/// Returns the current (highest active, non-EOL) release.
#[must_use]
pub fn current_release(schedule: &NodejsReleaseSchedule) -> Option<&NodejsRelease> {
    supported_releases(schedule).last().copied()
}

/// Returns the active LTS release (past LTS date, before maintenance).
#[must_use]
pub fn active_lts_release(schedule: &NodejsReleaseSchedule) -> Option<&NodejsRelease> {
    let today = OffsetDateTime::now_utc().date();
    supported_releases(schedule).into_iter().rfind(|r| {
        r.metadata.lts.is_some_and(|lts| lts <= today)
            && r.metadata.maintenance.is_some_and(|m| today < m)
    })
}

// Returns releases that are currently supported (started and not yet EOL).
// Relies on releases being ordered ascending by version (matching upstream).
fn supported_releases(schedule: &NodejsReleaseSchedule) -> Vec<&NodejsRelease> {
    let today = OffsetDateTime::now_utc().date();
    schedule
        .releases
        .iter()
        .filter(|r| r.metadata.start <= today && today < r.end_of_life)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    #[test]
    fn parse_handles_latest() {
        let result = Range::parse("latest");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("*", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_exact_versions() {
        let result = Range::parse("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_starts_with_v() {
        let result = Range::parse("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_semver_semantics() {
        let result = Range::parse(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_pipe_statements() {
        let result = Range::parse("^12 || ^13 || ^14");

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
        let result = Range::parse("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals_and_patch() {
        let result = Range::parse("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_v_within_string() {
        let result = Range::parse(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_v_with_space() {
        let result = Range::parse(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_handles_equal_with_v() {
        let result = Range::parse("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{}", reqs.range));
        }
    }

    #[test]
    fn parse_returns_error_for_invalid_reqs() {
        let result = Range::parse("12.%");
        assert!(result.is_err());
    }

    fn days_from_now(days: i64) -> Date {
        (OffsetDateTime::now_utc() + Duration::days(days)).date()
    }

    fn release(
        range: &str,
        start: Date,
        end_of_life: Date,
        lts: Option<Date>,
        maintenance: Option<Date>,
    ) -> NodejsRelease {
        release_schedule::Release {
            range: Range::parse(range).unwrap(),
            end_of_life,
            metadata: NodejsReleaseMetadata {
                start,
                lts,
                maintenance,
            },
        }
    }

    fn test_schedule() -> NodejsReleaseSchedule {
        NodejsReleaseSchedule {
            releases: vec![
                // v18: EOL (ended 100 days ago)
                release("v18", days_from_now(-1500), days_from_now(-100), None, None),
                // v20: maintenance LTS
                release(
                    "v20",
                    days_from_now(-1000),
                    days_from_now(200),
                    Some(days_from_now(-800)),
                    Some(days_from_now(-300)),
                ),
                // v24: active LTS (past lts, before maintenance)
                release(
                    "v24",
                    days_from_now(-400),
                    days_from_now(800),
                    Some(days_from_now(-180)),
                    Some(days_from_now(200)),
                ),
                // v25: current (highest active, no lts)
                release("v25", days_from_now(-100), days_from_now(300), None, None),
                // v27: not yet started
                release("v27", days_from_now(100), days_from_now(1000), None, None),
            ],
        }
    }

    #[test]
    fn toml_round_trip() {
        let toml_str = r#"
[[releases]]
range = "v24"
end_of_life = "2028-04-30"

[releases.metadata]
start = "2025-04-22"
lts = "2025-10-28"
maintenance = "2026-10-20"

[[releases]]
range = "v25"
end_of_life = "2026-06-01"

[releases.metadata]
start = "2025-04-22"
"#;
        let parsed: NodejsReleaseSchedule = toml_str.parse().expect("should parse");
        assert_eq!(parsed.releases.len(), 2);
        assert_eq!(parsed.releases[0].range.to_string(), "v24");
        assert_eq!(parsed.releases[1].range.to_string(), "v25");
    }

    #[test]
    fn lookup_matches_version_to_release() {
        let schedule = test_schedule();

        let release = schedule.lookup(&Version::parse("24.1.0").unwrap());
        assert_eq!(release.unwrap().range.to_string(), "v24");

        let release = schedule.lookup(&Version::parse("18.20.0").unwrap());
        assert_eq!(release.unwrap().range.to_string(), "v18");
    }

    #[test]
    fn lookup_returns_none_for_prerelease() {
        let schedule = test_schedule();
        assert!(
            schedule
                .lookup(&Version::parse("24.1.0-alpha.1").unwrap())
                .is_none()
        );
    }

    #[test]
    fn lookup_returns_none_for_unknown_version() {
        let schedule = test_schedule();
        assert!(
            schedule
                .lookup(&Version::parse("99.0.0").unwrap())
                .is_none()
        );
    }

    #[test]
    fn release_is_eol() {
        let schedule = test_schedule();

        // v18 eol was 100 days ago
        let v18 = schedule
            .lookup(&Version::parse("18.20.0").unwrap())
            .unwrap();
        assert!(is_eol(v18));

        // v24 eol in the future
        let v24 = schedule.lookup(&Version::parse("24.1.0").unwrap()).unwrap();
        assert!(!is_eol(v24));
    }

    #[test]
    fn current_returns_highest_active_release() {
        let schedule = test_schedule();
        assert_eq!(
            current_release(&schedule).map(|r| r.range.to_string()),
            Some("v25".to_string())
        );
    }

    #[test]
    fn active_lts_returns_lts_before_maintenance() {
        let schedule = test_schedule();
        assert_eq!(
            active_lts_release(&schedule).map(|r| r.range.to_string()),
            Some("v24".to_string())
        );
    }
}
