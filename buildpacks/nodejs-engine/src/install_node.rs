use bullet_stream::state::Bullet;
use bullet_stream::{style, Print};
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libherokubuildpack::download::download_file;
use libherokubuildpack::fs::move_directory_contents;
use libherokubuildpack::inventory::artifact::Artifact;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Stdout};
use std::path::Path;
use tempfile::NamedTempFile;
use thiserror::Error;

use heroku_nodejs_utils::vrs::Version;

use crate::{NodeJsEngineBuildpack, NodeJsEngineBuildpackError};

pub(crate) fn install_node(
    context: &BuildContext<NodeJsEngineBuildpack>,
    distribution_artifact: &Artifact<Version, Sha256, Option<()>>,
    log: Print<Bullet<Stdout>>,
) -> Result<Print<Bullet<Stdout>>, libcnb::Error<NodeJsEngineBuildpackError>> {
    let mut bullet = log.bullet("Installing Node.js distribution");

    let new_metadata = DistLayerMetadata {
        artifact: distribution_artifact.clone(),
        layer_version: LAYER_VERSION.to_string(),
    };

    let distribution_layer = context.cached_layer(
        layer_name!("dist"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &DistLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    let version_tag = format!(
        "{} ({}-{})",
        distribution_artifact.version, distribution_artifact.os, distribution_artifact.arch
    );
    match distribution_layer.state {
        LayerState::Restored { .. } => {
            bullet = bullet.sub_bullet(format!("Reusing Node.js {version_tag}"));
        }
        LayerState::Empty { .. } => {
            distribution_layer.write_metadata(new_metadata)?;

            let node_tgz = NamedTempFile::new().map_err(DistLayerError::TempFile)?;

            let timer = bullet.start_timer(format!(
                "Downloading Node.js {} from {}",
                style::value(&version_tag),
                style::url(&distribution_artifact.url)
            ));
            download_file(&distribution_artifact.url, node_tgz.path())
                .map_err(DistLayerError::Download)?;
            bullet = timer.done();

            bullet = bullet.sub_bullet("Verifying checksum");
            let digest = sha256(node_tgz.path()).map_err(DistLayerError::ReadTempFile)?;
            if distribution_artifact.checksum.value != digest {
                Err(DistLayerError::ChecksumVerification)?;
            }

            bullet =
                bullet.sub_bullet(format!("Extracting Node.js {}", style::value(&version_tag)));
            decompress_tarball(&mut node_tgz.into_file(), distribution_layer.path())
                .map_err(DistLayerError::Untar)?;

            let timer =
                bullet.start_timer(format!("Installing Node.js {}", style::value(&version_tag)));
            let dist_name = extract_tarball_prefix(&distribution_artifact.url)
                .ok_or_else(|| DistLayerError::TarballPrefix(distribution_artifact.url.clone()))?;
            let dist_path = distribution_layer.path().join(dist_name);
            move_directory_contents(dist_path, distribution_layer.path())
                .map_err(DistLayerError::Installation)?;
            bullet = timer.done();
        }
    };

    Ok(bullet.done())
}

fn sha256(path: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
    let mut file = fs::File::open(path.as_ref())?;
    let mut buffer = [0x00; 10 * 1024];
    let mut sha256: Sha256 = Sha256::default();

    let mut read = file.read(&mut buffer)?;
    while read > 0 {
        Digest::update(&mut sha256, &buffer[..read]);
        read = file.read(&mut buffer)?;
    }

    Ok(sha256.finalize().to_vec())
}

fn extract_tarball_prefix(url: &str) -> Option<&str> {
    url.rfind('/').and_then(|last_slash| {
        url.rfind(".tar.gz")
            .map(|tar_gz_index| &url[last_slash + 1..tar_gz_index])
    })
}

const LAYER_VERSION: &str = "1";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DistLayerMetadata {
    artifact: Artifact<Version, Sha256, Option<()>>,
    layer_version: String,
}

#[derive(Error, Debug)]
pub(crate) enum DistLayerError {
    #[error("Couldn't create tempfile for Node.js distribution: {0}")]
    TempFile(std::io::Error),
    #[error("Couldn't download Node.js distribution: {0}")]
    Download(libherokubuildpack::download::DownloadError),
    #[error("Couldn't decompress Node.js distribution: {0}")]
    Untar(std::io::Error),
    #[error("Couldn't extract tarball prefix from artifact URL: {0}")]
    TarballPrefix(String),
    #[error("Couldn't move Node.js distribution artifacts to the correct location: {0}")]
    Installation(std::io::Error),
    #[error("Error verifying checksum")]
    ChecksumVerification,
    #[error("Couldn't read tempfile for Node.js distribution: {0}")]
    ReadTempFile(std::io::Error),
}

impl From<DistLayerError> for libcnb::Error<NodeJsEngineBuildpackError> {
    fn from(value: DistLayerError) -> Self {
        libcnb::Error::BuildpackError(NodeJsEngineBuildpackError::DistLayerError(value))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use libherokubuildpack::inventory::{
        artifact::{Arch, Os},
        checksum::Checksum,
    };

    use super::*;

    #[test]
    fn dist_metadata_sanity_check() {
        let node_version_22_1_0_linux_arm = Artifact {
            version: Version::from_str("22.1.0").unwrap(),
            os: Os::Linux,
            arch: Arch::Arm64,
            url: "https://nodejs.org/download/release/v22.1.0/node-v22.1.0-linux-arm64.tar.gz"
                .to_string(),
            checksum: "sha256:9c111af1f951e8869615bca3601ce7ab6969374933bdba6397469843b808f222"
                .parse::<Checksum<Sha256>>()
                .unwrap(),
            metadata: None,
        };
        let node_version_22_1_0_linux_amd = Artifact {
            version: Version::from_str("22.1.0").unwrap(),
            os: Os::Linux,
            arch: Arch::Amd64,
            url: "https://nodejs.org/download/release/v22.1.0/node-v22.1.0-linux-x64.tar.gz"
                .to_string(),
            checksum: "sha256:d8ae35a9e2bb0c0c0611ee9bacf564ea51cc8291ace1447f95ee6aeaf4f1d61d"
                .parse::<Checksum<Sha256>>()
                .unwrap(),
            metadata: None,
        };

        // this is a check to ensure that the same node.js artifact doesn't invalidate the cache
        assert_eq!(
            DistLayerMetadata {
                artifact: node_version_22_1_0_linux_arm.clone(),
                layer_version: LAYER_VERSION.to_string()
            },
            DistLayerMetadata {
                artifact: node_version_22_1_0_linux_arm.clone(),
                layer_version: LAYER_VERSION.to_string()
            }
        );

        // this is a check to ensure that a different node.js artifact does invalidate the cache
        assert_ne!(
            DistLayerMetadata {
                artifact: node_version_22_1_0_linux_arm,
                layer_version: LAYER_VERSION.to_string()
            },
            DistLayerMetadata {
                artifact: node_version_22_1_0_linux_amd,
                layer_version: LAYER_VERSION.to_string()
            }
        );
    }

    #[test]
    fn test_metadata_guard() {
        let metadata = DistLayerMetadata {
            artifact: Artifact {
                version: Version::from_str("22.1.0").unwrap(),
                os: Os::Linux,
                arch: Arch::Amd64,
                url: "https://nodejs.org/download/release/v22.1.0/node-v22.1.0-linux-x64.tar.gz"
                    .to_string(),
                checksum: "sha256:d8ae35a9e2bb0c0c0611ee9bacf564ea51cc8291ace1447f95ee6aeaf4f1d61d"
                    .parse::<Checksum<Sha256>>()
                    .unwrap(),
                metadata: None,
            },
            layer_version: LAYER_VERSION.to_string(),
        };
        let actual = toml::to_string(&metadata).unwrap();
        let expected = r#"
layer_version = "1"

[artifact]
version = "22.1.0"
os = "linux"
arch = "amd64"
url = "https://nodejs.org/download/release/v22.1.0/node-v22.1.0-linux-x64.tar.gz"
checksum = "sha256:d8ae35a9e2bb0c0c0611ee9bacf564ea51cc8291ace1447f95ee6aeaf4f1d61d"
"#
        .trim();
        assert_eq!(expected, actual.trim());
        let from_toml: DistLayerMetadata = toml::from_str(&actual).unwrap();
        assert_eq!(metadata, from_toml);
    }
}
