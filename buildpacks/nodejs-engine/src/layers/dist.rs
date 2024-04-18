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
use sha2::Sha256;
use std::path::Path;
use tempfile::NamedTempFile;
use thiserror::Error;

/// A layer that downloads the Node.js distribution artifacts
pub(crate) struct DistLayer {
    pub(crate) artifact: Artifact<Version, Sha256>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct DistLayerMetadata {
    layer_version: String,
    nodejs_version: String,
    arch: String,
    os: String,
}

#[derive(Error, Debug)]
pub(crate) enum DistLayerError {
    #[error("Couldn't create tempfile for Node.js distribution: {0}")]
    TempFile(std::io::Error),
    #[error("Couldn't download Node.js distribution: {0}")]
    Download(libherokubuildpack::download::DownloadError),
    #[error("Couldn't decompress Node.js distribution: {0}")]
    Untar(std::io::Error),
    #[error("Couldn't move Node.js distribution artifacts to the correct location: {0}")]
    Installation(std::io::Error),
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
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsEngineBuildpackError> {
        let node_tgz = NamedTempFile::new().map_err(DistLayerError::TempFile)?;

        log_info(format!("Downloading Node.js {}", self.artifact.version));
        download_file(&self.artifact.url, node_tgz.path()).map_err(DistLayerError::Download)?;

        log_info(format!("Extracting Node.js {}", self.artifact.version));
        decompress_tarball(&mut node_tgz.into_file(), layer_path).map_err(DistLayerError::Untar)?;

        log_info(format!("Installing Node.js {}", self.artifact.version));
        let dist_name = format!("node-v{}-{}", self.artifact.version, "linux-x64");
        let dist_path = Path::new(layer_path).join(dist_name);
        move_directory_contents(dist_path, layer_path).map_err(DistLayerError::Installation)?;

        LayerResultBuilder::new(DistLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &mut self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata == DistLayerMetadata::current(self, context) {
            log_info(format!("Reusing Node.js {}", self.artifact.version));
            Ok(ExistingLayerStrategy::Keep)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl DistLayerMetadata {
    fn current(layer: &DistLayer, context: &BuildContext<NodeJsEngineBuildpack>) -> Self {
        DistLayerMetadata {
            nodejs_version: layer.artifact.version.to_string(),
            layer_version: String::from(LAYER_VERSION),
            arch: context.target.arch.clone(),
            os: context.target.os.clone(),
        }
    }
}
