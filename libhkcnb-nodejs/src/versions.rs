use node_semver::{Range, Version};
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

/// Heroku nodebin AWS S3 Bucket name
pub const BUCKET: &str = "heroku-nodebin";
/// Heroku nodebin AWS S3 Region
pub const REGION: &str = "us-east-1";

/// Default/assumed operating system for node release lookups
#[cfg(target_os = "macos")]
pub const OS: &str = "darwin";
#[cfg(target_os = "linux")]
pub const OS: &str = "linux";

/// Default/assumed architecture for node release lookups
#[cfg(target_arch = "x86_64")]
pub const ARCH: &str = "x64";

/// Represents a software inventory with releases.
#[derive(Debug, Deserialize, Serialize)]
pub struct Inventory {
    pub name: String,
    pub releases: Vec<Release>,
}

impl Inventory {
    /// Resolves the `Release` based on `semver-node::Range`.
    /// If no Release can be found, then `None` is returned.
    #[must_use]
    pub fn resolve(&self, req: &Req) -> Option<&Release> {
        let platform = format!("{}-{}", OS, ARCH);
        self.resolve_other(req, &platform, "release")
    }

    #[must_use]
    pub fn resolve_other(
        &self,
        version_requirements: &Req,
        platform: &str,
        channel: &str,
    ) -> Option<&Release> {
        let mut filtered_versions: Vec<&Release> = self
            .releases
            .iter()
            .filter(|version| {
                version.arch.as_deref().unwrap_or(platform) == platform
                    && version.channel == channel
            })
            .collect();
        // reverse sort, so latest is at the top
        filtered_versions.sort_by(|a, b| b.version.0.cmp(&a.version.0));

        filtered_versions
            .into_iter()
            .find(|rel| version_requirements.satisfies(&rel.version))
    }
}

/// Represents a inv release.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Release {
    pub version: Ver,
    pub channel: String,
    pub arch: Option<String>,
    pub url: String,
    pub etag: String,
}

#[derive(Debug)]
pub struct VerErr(String);
impl Error for VerErr {}
impl fmt::Display for VerErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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

    fn url(version: &str, arch: &str, channel: &str) -> String {
        format!(
            "https://s3.amazonaws.com/heroku-nodebin/node/{}/{}/node-v{}-{}.tar.gz",
            channel, arch, version, arch
        )
    }

    fn release(ver: Ver, arch: &str, channel: &str) -> Release {
        Release {
            version: ver.clone(),
            channel: channel.to_string(),
            arch: Some(arch.to_string()),
            url: url(&ver.to_string(), arch, channel),
            etag: "a586044d93acb053d28dd6c0ddf95362".to_string(),
        }
    }

    fn create_inventory() -> Inventory {
        let versions = vec![
            "13.10.0", "13.10.1", "13.11.0", "13.12.0", "13.13.0", "13.14.0", "14.0.0", "15.0.0",
        ];
        let mut releases = vec![];
        for v in versions {
            releases.push(release(Ver::parse(v).unwrap(), "linux-x64", "release"));
            releases.push(release(Ver::parse(v).unwrap(), "darwin-x64", "release"));
        }
        Inventory {
            name: "node".to_string(),
            releases,
        }
    }

    #[test]
    fn resolve_other_resolves_based_on_arch_and_channel() {
        let inv = create_inventory();
        let version_req = Req::parse("*").unwrap();

        let option = inv.resolve_other(version_req, "linux-x64", "release");
        assert!(option.is_some());
        if let Some(release) = option {
            assert_eq!("15.0.0", format!("{}", release.version));
            assert_eq!("linux-x64", release.arch.as_ref().unwrap());
            assert_eq!("release", release.channel);
        }
    }

    #[test]
    fn resolve_other_handles_x_in_version_requirement() {
        let inv = create_inventory();
        let version_req = Req::parse("13.10.x").unwrap();

        let option = inv.resolve_other(version_req, "linux-x64", "release");
        assert!(option.is_some());
        if let Some(release) = option {
            assert_eq!("13.10.1", format!("{}", release.version));
            assert_eq!("linux-x64", release.arch.as_ref().unwrap());
            assert_eq!("release", release.channel);
        }
    }

    #[test]
    fn resolve_returns_none_if_no_valid_version() {
        let inv = create_inventory();
        let version_req = Req::parse("18.0.0").unwrap();

        let option = inv.resolve(version_req);
        assert!(option.is_none());
    }

    #[test]
    fn resolve_handles_semver_from_apps() {
        let versions = vec![
            "10.0.0", "10.1.0", "10.10.0", "10.11.0", "10.12.0", "10.13.0", "10.14.0", "10.14.1",
            "10.14.2", "10.15.0", "10.15.1", "10.15.2", "10.15.3", "10.2.0", "10.2.1", "10.3.0",
            "10.4.0", "10.4.1", "10.5.0", "10.6.0", "10.7.0", "10.8.0", "10.9.0", "11.0.0",
            "11.1.0", "11.10.0", "11.10.1", "11.11.0", "11.12.0", "11.13.0", "11.14.0", "11.2.0",
            "11.3.0", "11.4.0", "11.5.0", "11.6.0", "11.7.0", "11.8.0", "11.9.0", "6.0.0", "6.1.0",
            "6.10.0", "6.10.1", "6.10.2", "6.10.3", "6.11.0", "6.11.1", "6.11.2", "6.11.3",
            "6.11.4", "6.11.5", "6.12.0", "6.12.1", "6.12.2", "6.12.3", "6.13.0", "6.13.1",
            "6.14.0", "6.14.1", "6.14.2", "6.14.3", "6.14.4", "6.15.0", "6.15.1", "6.16.0",
            "6.17.0", "6.17.1", "6.2.0", "6.2.1", "6.2.2", "6.3.0", "6.3.1", "6.4.0", "6.5.0",
            "6.6.0", "6.7.0", "6.8.0", "6.8.1", "6.9.0", "6.9.1", "6.9.2", "6.9.3", "6.9.4",
            "6.9.5", "8.0.0", "8.1.0", "8.1.1", "8.1.2", "8.1.3", "8.1.4", "8.10.0", "8.11.0",
            "8.11.1", "8.11.2", "8.11.3", "8.11.4", "8.12.0", "8.13.0", "8.14.0", "8.14.1",
            "8.15.0", "8.15.1", "8.16.0", "8.2.0", "8.2.1", "8.3.0", "8.4.0", "8.5.0", "8.6.0",
            "8.7.0", "8.8.0", "8.8.1", "8.9.0", "8.9.1", "8.9.2", "8.9.3", "8.9.4",
        ];
        let mut releases = vec![];
        for v in versions {
            releases.push(release(Ver::parse(v).unwrap(), "linux-x64", "release"));
            releases.push(release(Ver::parse(v).unwrap(), "darwin-x64", "release"));
        }

        let inv = Inventory {
            name: "node".to_string(),
            releases,
        };

        for (input, version) in vec![
            ("10.x", "10.15.3"),
            ("10.*", "10.15.3"),
            ("10", "10.15.3"),
            ("8.x", "8.16.0"),
            ("^8.11.3", "8.16.0"),
            ("~8.11.3", "8.11.4"),
            (">= 6.0.0", "11.14.0"),
            ("^6.9.0 || ^8.9.0 || ^10.13.0", "10.15.3"),
            ("6.* || 8.* || >= 10.*", "11.14.0"),
            (">= 6.11.1 <= 10", "8.16.0"),
            (">=8.10 <11", "10.15.3"),
        ]
        .iter()
        {
            let version_req = Req::parse(input).unwrap();
            let option = inv.resolve(version_req.clone());

            println!("vr: {}", version_req);
            assert!(option.is_some());

            println!("rv: {:?}", option.unwrap());
            if let Some(release) = option {
                assert_eq!(version, &format!("{}", release.version));
                assert_eq!("release", release.channel);
            }
        }
    }

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
