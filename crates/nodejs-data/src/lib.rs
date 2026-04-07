use chrono::{DateTime, NaiveDate, Utc};
use fun_run::NamedOutput;
use libherokubuildpack::inventory::version::VersionRequirement;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

pub struct NodeRelease {
    pub requirement: VersionRange,
    pub end_of_life: NaiveDate,
    pub lts_start: Option<NaiveDate>,
}

impl NodeRelease {
    #[must_use]
    pub fn is_eol(&self, as_of: DateTime<Utc>) -> bool {
        self.end_of_life < as_of.date_naive()
    }
}

pub struct NodeReleaseSchedule {
    releases: Vec<NodeRelease>,
}

impl NodeReleaseSchedule {
    pub fn from_json(json: &str) -> Result<Self, String> {
        #[derive(Deserialize)]
        struct ReleaseDates {
            #[serde(rename = "end")]
            end_of_life: NaiveDate,
            #[serde(rename = "lts")]
            lts_start: Option<NaiveDate>,
        }

        let map: BTreeMap<String, ReleaseDates> =
            serde_json::from_str(json).map_err(|e| e.to_string())?;

        let releases = map
            .into_iter()
            .map(|(key, dates)| {
                let requirement = VersionRange::parse(&key)
                    .map_err(|e| format!("Invalid schedule key '{key}': {e}"))?;
                Ok(NodeRelease {
                    requirement,
                    end_of_life: dates.end_of_life,
                    lts_start: dates.lts_start,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(NodeReleaseSchedule { releases })
    }

    #[must_use]
    pub fn resolve(&self, version: &Version) -> Option<&NodeRelease> {
        self.releases
            .iter()
            .find(|release| release.requirement.satisfies(version))
    }

    pub fn supported_lts(&self, as_of: DateTime<Utc>) -> impl Iterator<Item = &NodeRelease> {
        self.releases.iter().filter(move |release| {
            !release.is_eol(as_of)
                && release
                    .lts_start
                    .is_some_and(|lts| lts <= as_of.date_naive())
        })
    }

    // Releases are ordered by the BTreeMap key from the schedule JSON (e.g. "v18", "v20", ...),
    // so the last supported LTS entry is the newest. Note: this lexicographic ordering only
    // works correctly for consistently-sized major version numbers (e.g. not v0.8 and v0.10,
    // but those won't be displayed in any of the supported LTS releases anyway).
    #[must_use]
    pub fn newest_supported_lts(&self, as_of: DateTime<Utc>) -> Option<&NodeRelease> {
        self.supported_lts(as_of).last()
    }

    #[must_use]
    pub fn supported_lts_labels(&self, as_of: DateTime<Utc>) -> Vec<String> {
        self.supported_lts(as_of)
            .map(|release| release.requirement.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

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

    fn test_schedule() -> NodeReleaseSchedule {
        NodeReleaseSchedule::from_json(
            r#"{
                "v18": { "lts": "2022-10-25", "end": "2025-04-30" },
                "v20": { "lts": "2023-10-24", "end": "2026-04-30" },
                "v21": { "end": "2024-06-01" }
            }"#,
        )
        .unwrap()
    }

    #[test]
    fn resolve_finds_match() {
        let schedule = test_schedule();
        let release = schedule
            .resolve(&Version::parse("20.11.0").unwrap())
            .unwrap();
        assert_eq!(release.requirement.to_string(), "v20");
    }

    #[test]
    fn resolve_returns_none_for_unknown() {
        let schedule = test_schedule();
        assert!(
            schedule
                .resolve(&Version::parse("99.0.0").unwrap())
                .is_none()
        );
    }

    #[test]
    fn supported_lts_labels_includes_active_lts() {
        let schedule = test_schedule();
        assert_eq!(
            schedule.supported_lts_labels(Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap()),
            vec!["v18", "v20"]
        );
    }

    #[test]
    fn supported_lts_labels_excludes_pre_lts() {
        let schedule = test_schedule();
        // v20 LTS starts 2023-10-24 — pick a date before that
        assert_eq!(
            schedule.supported_lts_labels(Utc.with_ymd_and_hms(2023, 7, 1, 0, 0, 0).unwrap()),
            vec!["v18"]
        );
    }

    #[test]
    fn supported_lts_labels_excludes_eol() {
        let schedule = test_schedule();
        assert_eq!(
            schedule.supported_lts_labels(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
            vec!["v20"]
        );
    }

    #[test]
    fn eol_boundary_uses_utc_date() {
        let schedule = NodeReleaseSchedule::from_json(
            r#"{ "v99": { "lts": "2020-06-01", "end": "2025-06-15" } }"#,
        )
        .unwrap();
        let release = schedule
            .resolve(&Version::parse("99.0.0").unwrap())
            .unwrap();

        let before_eol = Utc.with_ymd_and_hms(2025, 6, 14, 23, 59, 59).unwrap();
        let on_eol = Utc.with_ymd_and_hms(2025, 6, 15, 12, 0, 0).unwrap();
        let after_eol = Utc.with_ymd_and_hms(2025, 6, 16, 0, 0, 0).unwrap();

        // Before EOL date: not EOL, included in supported
        assert!(!release.is_eol(before_eol));
        assert_eq!(schedule.supported_lts_labels(before_eol), vec!["v99"]);

        // On the EOL date itself: not yet considered EOL
        assert!(!release.is_eol(on_eol));
        assert_eq!(schedule.supported_lts_labels(on_eol), vec!["v99"]);

        // The day after (UTC): EOL, excluded from supported
        assert!(release.is_eol(after_eol));
        assert!(schedule.supported_lts_labels(after_eol).is_empty());
    }
}
