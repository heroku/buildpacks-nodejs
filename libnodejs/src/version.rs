use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use node_semver::{Range, Version, SemverError};

/// Heroku nodebin AWS S3 Bucket name
pub const BUCKET: &str = "heroku-nodebin";
/// Heroku nodebin AWS S3 Region
pub const REGION: &str = "us-east-1";

/// Parses `package.json` version requirements into `semver-node::Range`. It 
/// handles these cases:
///
/// * "latest" as "*"
/// * "~=" as "="
/// * any other node semver compatible requirements
///
/// # Failures
/// Invalid versions wil return an error
pub fn parse(requirement: &str) -> Result<Range, SemverError> {
    let trimmed = requirement.trim();

    if requirement == "latest" {
        Ok(Range::any())
    } else if let Ok(range) = Range::parse(&trimmed) {
        Ok(range)
    } else if trimmed.starts_with("~=") {
        let version = trimmed.replacen("=", "", 1);
        Range::parse(version)
    } else {
        Range::parse(&trimmed)
    }
}

/// Represents a software with releases.
#[derive(Debug, Deserialize, Serialize)]
pub struct Software {
    pub name: String,
    pub releases: Vec<Release>,
}

impl Software {
    /// Resolves the [`Release`](struct.Release.html) based on `semver-node::Range'.
    /// If no Release can be found, then `None` is returned.
    pub fn resolve(
        &self,
        version_requirements: Range,
        arch: &str,
        channel: &str,
    ) -> Option<&Release> {
        let mut filtered_versions: Vec<&Release> = self
            .releases
            .iter()
            .filter(|version| {
                version.arch.as_deref().unwrap_or(arch) == arch && version.channel == channel
            })
            .collect();
        // reverse sort, so latest is at the top
        filtered_versions.sort_by(|a, b| b.version.cmp(&a.version));

        filtered_versions
            .into_iter()
            .find(|&version| version_requirements.satisfies(&version.version))
    }
}

/// Represents a software release.
#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub version: Version,
    pub channel: String,
    pub arch: Option<String>,
    pub url: String,
    pub etag: String,
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

    fn release(version: &str, arch: &str, channel: &str) -> Release {
        Release {
            version: Version::parse(version).unwrap(),
            channel: channel.to_string(),
            arch: Some(arch.to_string()),
            url: url(version, arch, channel),
            etag: "a586044d93acb053d28dd6c0ddf95362".to_string(),
        }
    }

    fn create_software() -> Software {
        Software {
            name: "node".to_string(),
            releases: vec![
                release("13.10.0", "linux-x64", "release"),
                release("13.10.1", "linux-x64", "release"),
                release("13.11.0", "linux-x64", "release"),
                release("13.12.0", "linux-x64", "release"),
                release("13.13.0", "linux-x64", "release"),
                release("13.14.0", "linux-x64", "release"),
                release("14.0.0", "linux-x64", "release"),
                release("15.0.0", "linux-x64", "staging"),
                release("15.0.0", "darwin-x64", "release"),
            ],
        }
    }

    #[test]
    fn resolve_resolves_based_on_arch_and_channel() {
        let software = create_software();
        let version_req = Range::any();

        let option = software.resolve(version_req, "linux-x64", "release");
        assert!(option.is_some());
        if let Some(release) = option {
            assert_eq!("14.0.0", format!("{}", release.version));
            assert_eq!("linux-x64", release.arch.as_ref().unwrap());
            assert_eq!("release", release.channel);
        }
    }

    #[test]
    fn resolve_handles_x_in_version_requirement() {
        let software = create_software();
        let version_req = Range::parse("13.10.x").unwrap();

        let option = software.resolve(version_req, "linux-x64", "release");
        assert!(option.is_some());
        if let Some(release) = option {
            assert_eq!("13.10.1", format!("{}", release.version));
            assert_eq!("linux-x64", release.arch.as_ref().unwrap());
            assert_eq!("release", release.channel);
        }
    }

    #[test]
    fn resolve_returns_none_if_no_valid_version() {
        let software = create_software();
        let version_req = Range::parse("15.0.0").unwrap();

        let option = software.resolve(version_req, "linux-x64", "release");
        assert!(option.is_none());
    }

    #[test]
    fn resolve_handles_semver_from_apps() {
        let releases: Vec<Release> = vec![
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
        ]
        .iter()
        .map(|v| release(v, "linux-x64", "release"))
        .collect();

        let software = Software {
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
            let version_req = Range::parse(input).unwrap();
            let option = software.resolve(version_req.clone(), "linux-x64", "release");

            assert!(option.is_some());

            println!("vr: {:?}, v: {:?}", version_req, option.unwrap());
            if let Some(release) = option {
                assert_eq!(version, &format!("{}", release.version));
                assert_eq!("linux-x64", release.arch.as_ref().unwrap());
                assert_eq!("release", release.channel);
            }
        }
    }

    #[test]
    fn parse_handles_latest() {
        let result = parse("latest");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("*", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_exact_versions() {
        let result = parse("14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_starts_with_v() {
        let result = parse("v14.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("14.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_semver_semantics() {
        let result = parse(">= 12.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=12.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_pipe_statements() {
        let result = parse("^12 || ^13 || ^14");

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
        let result = parse("~=14.4");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.0 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_tilde_with_equals_and_patch() {
        let result = parse("~=14.4.3");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=14.4.3 <14.5.0-0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_v_within_string() {
        let result = parse(">v15.5.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">15.5.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_v_with_space() {
        let result = parse(">= v10.0.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!(">=10.0.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_handles_equal_with_v() {
        let result = parse("=v10.22.0");

        assert!(result.is_ok());
        if let Ok(reqs) = result {
            assert_eq!("10.22.0", format!("{}", reqs));
        }
    }

    #[test]
    fn parse_returns_error_for_invalid_reqs() {
        let result = parse("12.%");
        println!("{:?}", result);

        assert!(result.is_err());
    }
}
