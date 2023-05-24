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
pub const DEFAULT_REGION: &str = "us-east-1";

#[derive(Debug, Clone, Copy)]
pub enum Distribution {
    Yarn,
    Node,
}

impl FromStr for Distribution {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Node.js" | "node" => Ok(Self::Node),
            "Yarn" | "yarn" => Ok(Self::Yarn),
            other => Err(anyhow!("Unknown distribution: {other}")),
        }
    }
}

impl fmt::Display for Distribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Node => write!(f, "Node.js"),
            Self::Yarn => write!(f, "Yarn"),
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
        }
    }
    fn active_requirement(self) -> anyhow::Result<Requirement> {
        Requirement::parse(match self {
            Self::Node => ">= 16",
            Self::Yarn => ">= 1.22",
        })
        .map_err(|e| anyhow!("{e}"))
    }

    pub(crate) fn mirrored_path_regex(self) -> anyhow::Result<Regex> {
        Regex::new(match self {
            Self::Node => r"node/(?P<channel>\w+)/(?P<arch>[\w-]+)/node-v(?P<version>\d+\.\d+\.\d+)[\w-]+\.tar\.gz",
            Self::Yarn => r"yarn/(?P<channel>\w+)/yarn-v(?P<version>\d+\.\d+\.\d+(-[\w\.]+)?)\.tar\.gz",
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

fn list_upstream_yarn_versions() -> anyhow::Result<VersionSet> {
    let mut vset = VersionSet::new();
    for pkg in ["yarn", "@yarnpkg/cli-dist"] {
        for release in npmjs_org::list_releases(pkg)? {
            vset.insert(release.version);
        }
    }
    Ok(vset)
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
}
