use crate::{
    inv::Inventory,
    nodejs_org, npmjs_org, s3,
    vrs::{Requirement, Version, VersionSet},
};
use anyhow::anyhow;
use regex::Regex;
use std::{fmt, str::FromStr};

/// Heroku nodebin AWS S3 Bucket name
pub const DEFAULT_BUCKET: &str = "heroku-nodebin";
/// Heroku nodebin AWS S3 Region
pub(crate) const DEFAULT_REGION: &str = "us-east-1";

#[derive(Debug, Clone, Copy)]
pub enum Distribution {
    Yarn,
    Node,
    Npm,
}

impl FromStr for Distribution {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Node.js" | "node" => Ok(Self::Node),
            "Yarn" | "yarn" => Ok(Self::Yarn),
            "Npm" | "npm" => Ok(Self::Npm),
            other => Err(anyhow!("Unknown distribution: {other}")),
        }
    }
}

impl fmt::Display for Distribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node => write!(f, "Node.js"),
            Self::Yarn => write!(f, "Yarn"),
            Self::Npm => write!(f, "npm"),
        }
    }
}

impl Distribution {
    /// List all versions of a distribution from it's upstream/source location.
    ///
    /// # Errors
    /// - http call failures
    /// - http response parsing errors
    pub fn upstream_versions(&self) -> anyhow::Result<VersionSet> {
        match self {
            Self::Node => list_upstream_node_versions(),
            Self::Yarn => list_upstream_yarn_versions(),
            Self::Npm => list_upstream_npm_versions(),
        }
    }

    /// List all versions of a distribution from it's mirrored location.
    ///
    /// # Errors
    /// - http call failures
    /// - http response parsing errors
    pub fn mirrored_versions(&self, bucket: &str) -> anyhow::Result<VersionSet> {
        s3::list_objects(bucket, DEFAULT_REGION, self.bucket_prefix())?.try_into()
    }

    /// Build an inventory from the releases listed in the mirrored location.
    ///
    /// # Errors
    /// - http call failures
    /// - http response parsing errors
    pub fn mirrored_inventory(&self, bucket: &str) -> anyhow::Result<Inventory> {
        s3::list_objects(bucket, DEFAULT_REGION, self.bucket_prefix())?.try_into()
    }

    /// Filter inactive distribution versions and return as a `VersionSet`.
    ///
    /// # Errors
    /// - Active semver range for the distribution is a valid version.
    pub fn filter_inactive_versions<'i, I>(&self, iter: I) -> anyhow::Result<VersionSet>
    where
        I: Iterator<Item = &'i Version>,
    {
        let req = self.active_requirement()?;
        Ok(iter.filter(|v| req.satisfies(v)).cloned().collect())
    }

    fn bucket_prefix(&self) -> &str {
        match self {
            Self::Node => "node",
            Self::Yarn => "yarn",
            Self::Npm => "npm",
        }
    }

    /// The range of versions considered active for mirroring purposes.
    fn active_requirement(self) -> anyhow::Result<Requirement> {
        Requirement::parse(match self {
            Self::Node => ">=16",
            Self::Yarn => ">=1.22 || >=4.0.0-rc.35",
            Self::Npm => ">=8",
        })
        .map_err(|e| anyhow!("{e}"))
    }

    pub(crate) fn mirrored_path_regex(self) -> anyhow::Result<Regex> {
        Regex::new(match self {
            Self::Node => r"node/(?P<channel>\w+)/(?P<arch>[\w-]+)/node-v(?P<version>\d+\.\d+\.\d+)[\w-]+\.tar\.gz",
            Self::Yarn => r"yarn/(?P<channel>\w+)/yarn-v(?P<version>\d+\.\d+\.\d+(-[\w\.]+)?)\.tar\.gz",
            Self::Npm => r"npm/(?P<channel>\w+)/npm-v(?P<version>\d+\.\d+\.\d+(-[\w\.]+)?)\.tar\.gz"
        }).map_err(|e| anyhow!("Mirrored release regex error: {e}"))
    }
}

fn list_upstream_node_versions() -> anyhow::Result<VersionSet> {
    nodejs_org::list_releases()?
        .into_iter()
        .map(|rls| {
            Version::parse(&rls.version).map_err(|e| {
                anyhow!("Couldn't parse upstream nodejs.org version as a version: {e}")
            })
        })
        .collect()
}

const IGNORE_YARN_VERSIONS: [&str; 1] = [
    // This version is ignored because all of the current 2.x versions are published by the `@yarnpkg/cli-dist`
    // module except for this one which is published by the `yarn` module. The layout of this package
    // differs from what we expect so instead of coding in some edge case handling when we install this
    // yarn version in the buildpack, we've decided to ignore it.
    //
    // There should be little user impact here because our Yarn inventory only controls the "global" binary
    // that is installed which acts as a wrapper to the actual version of Yarn used when building.
    // For Yarn 2+ projects, the actual Yarn version used is meant to be committed to the folder
    // `.yarn/release` and the global binary delegates all operations to the committed version.
    "2.4.3",
];

fn list_upstream_yarn_versions() -> anyhow::Result<VersionSet> {
    let mut vset = VersionSet::new();
    for pkg in ["yarn", "@yarnpkg/cli-dist"] {
        for release in npmjs_org::list_releases(pkg)? {
            let ignore_release = pkg == "yarn"
                && IGNORE_YARN_VERSIONS.contains(&release.version.to_string().as_str());
            if !ignore_release {
                vset.insert(release.version);
            }
        }
    }
    Ok(vset)
}

fn list_upstream_npm_versions() -> anyhow::Result<VersionSet> {
    npmjs_org::list_releases("npm").map(|releases| {
        releases
            .into_iter()
            .map(|release| release.version)
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upstream_versions_node() {
        let dist = Distribution::Node {};
        let expected_version = Version::parse("20.0.0").expect("Expected to parse a valid version");
        let versions = dist
            .upstream_versions()
            .expect("Expected to list upstream remote versions, but got an error");
        let actual_version = versions
            .get(&expected_version)
            .expect("Expected to find a matching version");
        assert_eq!(&expected_version, actual_version);
    }

    #[test]
    fn upstream_versions_yarn() {
        let dist = Distribution::Yarn {};
        let expected_version =
            Version::parse("1.22.17").expect("Expected to parse a valid version");
        let versions = dist
            .upstream_versions()
            .expect("Expected to list upstream remote versions, but got an error");
        let actual_version = versions
            .get(&expected_version)
            .expect("Expected to find a matching version");
        assert_eq!(&expected_version, actual_version);
    }

    #[test]
    fn upstream_versions_npm() {
        let dist = Distribution::Npm {};
        let expected_version = Version::parse("9.7.2").expect("Expected to parse a valid version");
        let versions = dist
            .upstream_versions()
            .expect("Expected to list upstream remote versions, but got an error");
        let actual_version = versions
            .get(&expected_version)
            .expect("Expected to find a matching version");
        assert_eq!(&expected_version, actual_version);
    }

    #[test]
    fn mirrored_versions_node() {
        let dist = Distribution::Node {};
        let expected_version = Version::parse("18.0.0").expect("Expected to parse a valid version");
        let versions = dist
            .mirrored_versions(DEFAULT_BUCKET)
            .expect("Expected to list upstream remote versions, but got an error");
        let actual_version = versions
            .get(&expected_version)
            .expect("Expected to find a matching version");
        assert_eq!(&expected_version, actual_version);
    }

    #[test]
    fn mirrored_versions_yarn() {
        let dist = Distribution::Yarn {};
        let expected_version = Version::parse("3.0.0").expect("Expected to parse a valid version");
        let versions = dist
            .mirrored_versions(DEFAULT_BUCKET)
            .expect("Expected to list upstream remote versions, but got an error");
        let actual_version = versions
            .get(&expected_version)
            .expect("Expected to find a matching version");
        assert_eq!(&expected_version, actual_version);
    }

    #[test]
    fn filter_inactive_yarn() {
        let versions = ["1.20.1", "1.22.19", "3.0.0-rc.1", "3.2.3", "4.0.0-rc.44"]
            .into_iter()
            .map(Version::parse)
            .collect::<Result<VersionSet, _>>()
            .expect("Expected to parse all valid versions");

        let filtered = Distribution::Yarn {}
            .filter_inactive_versions(versions.iter())
            .expect("Expected to filter versions without an error");

        let expected = ["1.22.19", "3.2.3", "4.0.0-rc.44"]
            .into_iter()
            .map(Version::parse)
            .collect::<Result<VersionSet, _>>()
            .expect("Expected to parse all valid versions");

        assert_eq!(expected, filtered);
    }

    #[test]
    fn filter_inactive_node() {
        let versions = ["0.10.1", "14.2.4", "18.3.0", "20.2.0"]
            .into_iter()
            .map(Version::parse)
            .collect::<Result<VersionSet, _>>()
            .expect("Expected to parse all valid versions");

        let filtered = Distribution::Node {}
            .filter_inactive_versions(versions.iter())
            .expect("Expected to filter versions without an error");

        let expected = ["18.3.0", "20.2.0"]
            .into_iter()
            .map(Version::parse)
            .collect::<Result<VersionSet, _>>()
            .expect("Expected to parse all valid versions");

        assert_eq!(expected, filtered);
    }
}
