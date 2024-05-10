use crate::{NodeJsEngineBuildpack, NodeJsEngineBuildpackError};
use heroku_inventory_utils::inv::Artifact;
use heroku_nodejs_utils::vrs::Version;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::download::download_file;
use libherokubuildpack::fs::move_directory_contents;
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;
use tempfile::NamedTempFile;
use thiserror::Error;

/// A layer that downloads the Node.js distribution artifacts
pub(crate) struct DistLayer {
    pub(crate) artifact: Artifact<Version, Sha256>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct DistLayerMetadata {
    artifact: Artifact<Version, Sha256>,
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

const LAYER_VERSION: &str = "1";

impl Layer for DistLayer {
    type Buildpack = NodeJsEngineBuildpack;
    type Metadata = DistLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn create(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsEngineBuildpackError> {
        let node_tgz = NamedTempFile::new().map_err(DistLayerError::TempFile)?;

        log_info(format!(
            "Downloading Node.js {} from {}",
            self.artifact, self.artifact.url
        ));
        download_file(&self.artifact.url, node_tgz.path()).map_err(DistLayerError::Download)?;

        log_info("Verifying checksum");
        let digest = sha256(node_tgz.path()).map_err(DistLayerError::ReadTempFile)?;
        if self.artifact.checksum.value != digest {
            Err(DistLayerError::ChecksumVerification)?;
        }

        log_info(format!("Extracting Node.js {}", self.artifact));
        decompress_tarball(&mut node_tgz.into_file(), layer_path).map_err(DistLayerError::Untar)?;

        log_info(format!("Installing Node.js {}", self.artifact));

        let dist_name = extract_tarball_prefix(&self.artifact.url)
            .ok_or_else(|| DistLayerError::TarballPrefix(self.artifact.url.clone()))?;
        let dist_path = Path::new(layer_path).join(dist_name);
        move_directory_contents(dist_path, layer_path).map_err(DistLayerError::Installation)?;

        LayerResultBuilder::new(DistLayerMetadata::current(self)).build()
    }

    fn existing_layer_strategy(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata == DistLayerMetadata::current(self) {
            log_info(format!("Reusing Node.js {}", self.artifact));
            Ok(ExistingLayerStrategy::Keep)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

fn sha256(path: impl AsRef<Path>) -> Result<Vec<u8>, std::io::Error> {
    let mut file = fs::File::open(path.as_ref())?;
    let mut buffer = [0x00; 10 * 1024];
    let mut sha256: Sha256 = sha2::Sha256::default();

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

impl DistLayerMetadata {
    fn current(layer: &DistLayer) -> Self {
        DistLayerMetadata {
            artifact: layer.artifact.clone(),
            layer_version: String::from(LAYER_VERSION),
        }
    }
}
