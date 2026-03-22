//! Generic release schedule types for tracking version lifecycle and end-of-life dates.
//!
//! Provides [`Schedule`] and [`Release`] types that complement
//! `libherokubuildpack::inventory` by tracking when versions reach end-of-life.
//! Uses the same [`VersionRequirement`] trait for version matching.

use libherokubuildpack::inventory::version::VersionRequirement;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// A collection of releases, each covering a range of versions.
#[derive(Debug, Serialize, Deserialize)]
pub struct Schedule<R, D, M> {
    pub releases: Vec<Release<R, D, M>>,
}

/// A single release covering versions that match `range`, with an end-of-life date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release<R, D, M> {
    pub range: R,
    pub end_of_life: D,
    pub metadata: M,
}

impl<R, D, M> Default for Schedule<R, D, M> {
    fn default() -> Self {
        Self { releases: vec![] }
    }
}

impl<R, D, M> Schedule<R, D, M> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, release: Release<R, D, M>) {
        self.releases.push(release);
    }
}

impl<R, D, M> Schedule<R, D, M> {
    /// Find the release for a given version, if one exists.
    ///
    /// Returns `None` if no release's range satisfies the version
    /// (e.g., pre-release versions, or versions not tracked in the schedule).
    pub fn lookup<V>(&self, version: &V) -> Option<&Release<R, D, M>>
    where
        R: VersionRequirement<V>,
    {
        self.releases.iter().find(|r| r.range.satisfies(version))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseScheduleError {
    #[error("TOML parsing error: {0}")]
    TomlError(toml::de::Error),
}

impl<R, D, M> std::str::FromStr for Schedule<R, D, M>
where
    R: DeserializeOwned,
    D: DeserializeOwned,
    M: DeserializeOwned,
{
    type Err = ParseScheduleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        toml::from_str(s).map_err(ParseScheduleError::TomlError)
    }
}

impl<R, D, M> std::fmt::Display for Schedule<R, D, M>
where
    R: Serialize,
    D: Serialize,
    M: Serialize,
{
    #[allow(clippy::unwrap_used)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&toml::to_string(self).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A simple prefix-matching range for testing.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct PrefixRange(String);

    impl VersionRequirement<String> for PrefixRange {
        fn satisfies(&self, version: &String) -> bool {
            version.starts_with(&self.0)
        }
    }

    impl From<&str> for PrefixRange {
        fn from(s: &str) -> Self {
            PrefixRange(s.to_string())
        }
    }

    fn make_release(range: &str, eol: &str) -> Release<PrefixRange, String, ()> {
        Release {
            range: range.into(),
            end_of_life: eol.to_string(),
            metadata: (),
        }
    }

    #[test]
    fn new_schedule_is_empty() {
        let schedule: Schedule<PrefixRange, String, ()> = Schedule::new();
        assert!(schedule.releases.is_empty());
    }

    #[test]
    fn push_adds_release() {
        let mut schedule: Schedule<PrefixRange, String, ()> = Schedule::new();
        schedule.push(make_release("v1", "2025-01-01"));
        assert_eq!(schedule.releases.len(), 1);
    }

    #[test]
    fn lookup_finds_matching_release() {
        let schedule = Schedule {
            releases: vec![
                make_release("v1", "2025-01-01"),
                make_release("v2", "2026-01-01"),
            ],
        };

        let release = schedule.lookup(&"v2.5.0".to_string());
        assert!(release.is_some());
        assert_eq!(release.unwrap().end_of_life, "2026-01-01");
    }

    #[test]
    fn lookup_returns_none_for_unmatched_version() {
        let schedule = Schedule {
            releases: vec![make_release("v1", "2025-01-01")],
        };
        assert!(schedule.lookup(&"v9.0.0".to_string()).is_none());
    }

    #[test]
    fn lookup_returns_first_match() {
        let schedule = Schedule {
            releases: vec![make_release("v1", "first"), make_release("v1", "second")],
        };
        let release = schedule.lookup(&"v1.0.0".to_string());
        assert_eq!(release.unwrap().end_of_life, "first");
    }

    #[test]
    fn toml_round_trip() {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        struct Meta {
            channel: String,
        }

        let toml_str = r#"
[[releases]]
range = "v1"
end_of_life = "2025-01-01"

[releases.metadata]
channel = "lts"

[[releases]]
range = "v2"
end_of_life = "2026-06-01"

[releases.metadata]
channel = "current"
"#;

        let parsed: Schedule<String, String, Meta> = toml_str.parse().expect("should parse");
        assert_eq!(parsed.releases.len(), 2);
        assert_eq!(parsed.releases[0].range, "v1");
        assert_eq!(parsed.releases[0].end_of_life, "2025-01-01");
        assert_eq!(parsed.releases[0].metadata.channel, "lts");
        assert_eq!(parsed.releases[1].range, "v2");
        assert_eq!(parsed.releases[1].metadata.channel, "current");

        // Round-trip
        let serialized = parsed.to_string();
        let reparsed: Schedule<String, String, Meta> = serialized.parse().expect("should re-parse");
        assert_eq!(reparsed.releases.len(), 2);
        assert_eq!(reparsed.releases[0].range, parsed.releases[0].range);
        assert_eq!(
            reparsed.releases[1].end_of_life,
            parsed.releases[1].end_of_life
        );
    }

    #[test]
    fn toml_round_trip_without_metadata() {
        let mut schedule = Schedule::<String, String, Option<()>>::new();
        schedule.push(Release {
            range: "v1".into(),
            end_of_life: "2025-01-01".into(),
            metadata: None,
        });

        let serialized = schedule.to_string();
        let reparsed: Schedule<String, String, Option<()>> =
            serialized.parse().expect("should re-parse");
        assert_eq!(reparsed.releases.len(), 1);
        assert_eq!(reparsed.releases[0].range, "v1");
    }
}
