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

pub type NodejsReleaseSchedule = BTreeMap<NodejsReleaseLine, NodejsRelease>;

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
        (other.major, other.minor.unwrap_or(0)).cmp(&(self.major, self.minor.unwrap_or(0)))
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

impl FromStr for NodejsReleaseLine {
    type Err = VersionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        NodejsReleaseLine::try_from(s.to_string())
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct NodejsRelease {
    #[serde(with = "toml_datetime_compat")]
    pub start: time::Date,
    #[serde(with = "toml_datetime_compat")]
    pub end: time::Date,
    #[serde(
        with = "toml_datetime_compat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub lts: Option<time::Date>,
    #[serde(
        with = "toml_datetime_compat",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub maintenance: Option<time::Date>,
}

/// Combined inventory and release schedule for Node.js.
#[derive(Debug, Serialize, Deserialize)]
pub struct NodejsInventoryWithSchedule {
    /// The release schedule containing lifecycle information for each release line.
    pub schedule: NodejsReleaseSchedule,
    /// The artifact inventory containing downloadable Node.js versions.
    #[serde(flatten)]
    pub inventory: NodejsInventory,
}

// --- Lifecycle computation functions ---

/// Major versions that should cause an immediate build failure when resolved.
/// Currently empty; versions here would be rejected outright during builds.
pub const REJECTED_VERSIONS: &[u64] = &[];

/// Returns the major version of the current (non-LTS) release line.
///
/// This is the highest-numbered release line in the schedule that has no `lts` date.
///
/// # Panics
///
/// Panics if no current (non-LTS) release line is found in the schedule.
#[must_use]
pub fn current_version(schedule: &NodejsReleaseSchedule) -> u64 {
    let today = time::OffsetDateTime::now_utc().date();
    schedule
        .iter()
        .filter(|(_, r)| r.lts.is_none() && r.end > today)
        .map(|(line, _)| line.major())
        .max()
        // Safe to expect — the `current_version_is_present` test verifies that the
        // real inventory always contains at least one current release line.
        .expect("Schedule should contain at least one current (non-LTS) release line")
}

/// Returns the major version of the active LTS release line.
///
/// The active LTS is the highest-numbered release line whose `lts` date has
/// passed and whose `maintenance` date either hasn't been set or hasn't
/// arrived yet.
///
/// # Panics
///
/// Panics if no active LTS release line is found in the schedule.
#[must_use]
pub fn active_lts_version(schedule: &NodejsReleaseSchedule) -> u64 {
    let today = time::OffsetDateTime::now_utc().date();
    schedule
        .iter()
        .filter(|(_, r)| {
            r.lts.is_some_and(|lts| lts <= today) && r.maintenance.is_none_or(|maint| maint > today)
        })
        .map(|(line, _)| line.major())
        .max()
        // Safe to expect — the `active_lts_version_is_present` test verifies that the
        // real inventory always contains at least one active LTS release line.
        .expect("Schedule should contain at least one active LTS release line")
}

/// Returns the major versions of all maintenance LTS release lines.
///
/// A maintenance LTS release line has an `lts` date, a `maintenance` date that
/// has already passed, and an EOL date still in the future.
#[must_use]
pub fn maintenance_lts_versions(schedule: &NodejsReleaseSchedule) -> Vec<u64> {
    let today = time::OffsetDateTime::now_utc().date();
    schedule
        .iter()
        .filter(|(_, r)| {
            r.lts.is_some() && r.maintenance.is_some_and(|maint| maint <= today) && r.end > today
        })
        .map(|(line, _)| line.major())
        .collect()
}

/// Returns true if the range satisfies versions across multiple major versions
/// present in the inventory.
#[must_use]
pub fn is_wide_range(
    requested_range: &VersionRange,
    inventory: &NodejsInventoryWithSchedule,
) -> bool {
    let mut majors = std::collections::HashSet::new();
    for artifact in &inventory.inventory.artifacts {
        if requested_range.satisfies(&artifact.version) {
            majors.insert(artifact.version.major());
        }
    }
    if majors.len() > 1 {
        return true;
    }
    // Check if the range would also satisfy a version one major above the highest
    // in the inventory. This handles cases like ">=25" where 25 is the highest
    // major available — the range is still wide even though no 26.x artifacts exist yet.
    if let Some(&highest) = majors.iter().max()
        && let Ok(next_major) = Version::parse(&format!("{}.0.0", highest + 1))
    {
        return requested_range.satisfies(&next_major);
    }
    false
}

/// Returns the highest version matching the active LTS major from the inventory,
/// or None if no LTS artifacts match the range.
#[must_use]
pub fn lts_upper_bound(
    requested_range: &VersionRange,
    inventory: &NodejsInventoryWithSchedule,
) -> Option<Version> {
    let lts_major = active_lts_version(&inventory.schedule);
    inventory
        .inventory
        .artifacts
        .iter()
        .filter(|a| a.version.major() == lts_major && requested_range.satisfies(&a.version))
        .map(|a| &a.version)
        .max()
        .cloned()
}

/// Returns the EOL date for the given version from the release schedule.
///
/// Uses `Schedule::resolve` with `NodejsReleaseLine`'s `VersionRequirement` impl,
/// which handles v0.9/v0.11 development release edge cases automatically.
///
/// # Panics
///
/// Panics if the version is not represented in the release schedule.
/// The `all_inventory_artifacts_have_eol_dates` test verifies this holds for every
/// artifact in the inventory.
#[must_use]
pub fn eol_date_for_version(
    version: &Version,
    inventory: &NodejsInventoryWithSchedule,
) -> time::Date {
    // Find the release line whose VersionRequirement matches this version.
    // Safe to unwrap — the `all_inventory_artifacts_have_eol_dates` test exhaustively
    // verifies that every artifact in the inventory has a matching release schedule entry.
    inventory
        .schedule
        .iter()
        .find(|(line, _)| line.satisfies(version))
        .map_or_else(
            || panic!("No release schedule entry found for Node.js {version}"),
            |(_, release)| release.end,
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use libherokubuildpack::inventory::artifact::{Arch, Os};
    use libherokubuildpack::inventory::checksum::Checksum;
    use sha2::Sha256;

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
        let lines: Vec<NodejsReleaseLine> = ["v26", "v25", "v4", "v0.12", "v0.10", "v0.8"]
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
[schedule]
v25 = { start = 2025-04-22, end = 2026-06-01 }
v24 = { start = 2025-04-22, end = 2028-04-30, lts = 2025-10-28, maintenance = 2026-10-20 }

[[artifacts]]
version = "25.0.0"
os = "linux"
arch = "amd64"
url = "https://nodejs.org/download/release/v25.0.0/node-v25.0.0-linux-x64.tar.gz"
checksum = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
"#;
        let parsed: NodejsInventoryWithSchedule =
            toml::from_str(toml_str).expect("should parse TOML");

        let expected = NodejsInventoryWithSchedule {
            schedule: BTreeMap::from_iter([
                (
                    NodejsReleaseLine::from_str("v25").unwrap(),
                    NodejsRelease {
                        start: time::Date::from_calendar_date(2025, time::Month::April, 22)
                            .unwrap(),
                        end: time::Date::from_calendar_date(2026, time::Month::June, 1).unwrap(),
                        lts: None,
                        maintenance: None,
                    },
                ),
                (
                    NodejsReleaseLine::from_str("v24").unwrap(),
                    NodejsRelease {
                        start: time::Date::from_calendar_date(2025, time::Month::April, 22)
                            .unwrap(),
                        end: time::Date::from_calendar_date(2028, time::Month::April, 30).unwrap(),
                        lts: Some(
                            time::Date::from_calendar_date(2025, time::Month::October, 28).unwrap(),
                        ),
                        maintenance: Some(
                            time::Date::from_calendar_date(2026, time::Month::October, 20).unwrap(),
                        ),
                    },
                ),
            ]),
            inventory: NodejsInventory {
                artifacts: vec![NodejsArtifact {
                    version: Version::from_str("25.0.0").unwrap(),
                    os: Os::Linux,
                    arch: Arch::Amd64,
                    url:
                        "https://nodejs.org/download/release/v25.0.0/node-v25.0.0-linux-x64.tar.gz"
                            .to_string(),
                    checksum:
                        "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                            .parse::<Checksum<Sha256>>()
                            .unwrap(),
                    metadata: None,
                }],
            },
        };
        assert_eq!(parsed.schedule, expected.schedule);
        assert_eq!(parsed.inventory.artifacts.len(), 1);
        assert_eq!(
            parsed.inventory.artifacts[0],
            expected.inventory.artifacts[0]
        );

        // Re-serialize and parse again to verify round-trip
        let reserialized = toml::to_string(&parsed).expect("should serialize");
        let reparsed: NodejsInventoryWithSchedule =
            toml::from_str(&reserialized).expect("should parse re-serialized TOML");
        assert_eq!(reparsed.schedule.len(), 2);
        assert_eq!(reparsed.inventory.artifacts.len(), 1);
    }

    // --- Lifecycle computation tests using real inventory ---

    fn load_inventory() -> NodejsInventoryWithSchedule {
        let toml_str = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../inventory/nodejs.toml"),
        )
        .expect("should read inventory file");
        toml::from_str(&toml_str).expect("should parse inventory")
    }

    #[test]
    fn is_wide_range_narrow() {
        let inv = load_inventory();
        let range = VersionRange::parse("24.x").unwrap();
        assert!(!is_wide_range(&range, &inv));
    }

    #[test]
    fn is_wide_range_wide() {
        let inv = load_inventory();
        let range = VersionRange::parse(">= 22").unwrap();
        assert!(is_wide_range(&range, &inv));
    }

    #[test]
    fn is_wide_range_detected_when_range_starts_at_highest_major() {
        let inv = load_inventory();
        let highest_major = inv
            .inventory
            .artifacts
            .iter()
            .map(|a| a.version.major())
            .max()
            .unwrap();
        let range = VersionRange::parse(&format!(">={highest_major}")).unwrap();
        assert!(is_wide_range(&range, &inv));
    }

    #[test]
    fn lts_upper_bound_returns_highest_lts_match() {
        let inv = load_inventory();
        let lts_major = active_lts_version(&inv.schedule);
        let range = VersionRange::parse(">= 22").unwrap();
        let result = lts_upper_bound(&range, &inv).expect("should find an LTS version");
        assert_eq!(result.major(), lts_major);
        // It should be the highest artifact for the LTS major
        let highest_lts = inv
            .inventory
            .artifacts
            .iter()
            .filter(|a| a.version.major() == lts_major)
            .map(|a| &a.version)
            .max()
            .unwrap();
        assert_eq!(&result, highest_lts);
    }

    #[test]
    fn eol_date_for_version_returns_correct_date() {
        let inv = load_inventory();
        let version = Version::parse("24.0.0").unwrap();
        let eol = eol_date_for_version(&version, &inv);
        assert_eq!(
            eol,
            time::Date::from_calendar_date(2028, time::Month::April, 30).unwrap()
        );
    }

    #[test]
    fn all_inventory_artifacts_have_eol_dates() {
        let inv = load_inventory();
        for artifact in &inv.inventory.artifacts {
            let eol = eol_date_for_version(&artifact.version, &inv);
            assert!(
                eol.year() > 0,
                "Expected a valid EOL date for {}, got {eol}",
                artifact.version
            );
        }
    }

    #[test]
    fn rejected_versions_does_not_include_current() {
        let version = Version::parse("24.0.0").unwrap();
        assert!(!REJECTED_VERSIONS.contains(&version.major()));
    }

    #[test]
    fn current_version_is_present() {
        let inv = load_inventory();
        assert_eq!(current_version(&inv.schedule), 25);
    }

    #[test]
    fn active_lts_version_is_present() {
        let inv = load_inventory();
        assert_eq!(active_lts_version(&inv.schedule), 24);
    }

    #[test]
    fn maintenance_lts_versions_are_correct() {
        let inv = load_inventory();
        let mut versions = maintenance_lts_versions(&inv.schedule);
        versions.sort_unstable();
        assert_eq!(versions, vec![20, 22]);
    }
}
