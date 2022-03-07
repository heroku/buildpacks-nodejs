use std::path::Path;

use tempfile::NamedTempFile;

use crate::{NodeJsEngineBuildpack, NodeJsEngineBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};

use libcnb::data::buildpack::StackId;
use libcnb::Buildpack;
use libherokubuildpack::{decompress_tarball, download_file, log_info, move_directory_contents};
use libhkcnb_nodejs::inv::Release;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A layer that downloads the Node.js distribution artifacts
pub struct DistLayer {
    pub release: Release,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DistLayerMetadata {
    layer_version: String,
    nodejs_version: String,
    stack_id: StackId,
}

#[derive(Error, Debug)]
pub enum DistLayerError {
    #[error("Couldn't create tempfile for Node.js distribution: {0}")]
    TempFile(std::io::Error),
    #[error("Couldn't download Node.js distribution: {0}")]
    Download(libherokubuildpack::DownloadError),
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
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsEngineBuildpackError> {
        let node_tgz = NamedTempFile::new().map_err(DistLayerError::TempFile)?;

        log_info(format!("Downloading Node.js {}", self.release.version));
        download_file(&self.release.url, node_tgz.path()).map_err(DistLayerError::Download)?;

        log_info(format!("Extracting Node.js {}", self.release.version));
        decompress_tarball(&mut node_tgz.into_file(), &layer_path)
            .map_err(DistLayerError::Untar)?;

        log_info(format!("Installing Node.js {}", self.release.version));
        let dist_name = format!("node-v{}-{}", self.release.version, "linux-x64");
        let dist_path = Path::new(layer_path).join(dist_name);
        move_directory_contents(dist_path, layer_path).map_err(DistLayerError::Installation)?;

        let metadata = DistLayerMetadata {
            layer_version: LAYER_VERSION.to_string(),
            nodejs_version: self.release.version.to_string(),
            stack_id: context.stack_id.clone(),
        };

        LayerResultBuilder::new(metadata).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        let metadata = &layer_data.content_metadata.metadata;

        if self.release.version.to_string() != metadata.nodejs_version {
            return Ok(ExistingLayerStrategy::Recreate);
        }

        if context.stack_id != metadata.stack_id {
            return Ok(ExistingLayerStrategy::Recreate);
        }

        if LAYER_VERSION != metadata.layer_version {
            return Ok(ExistingLayerStrategy::Recreate);
        }

        log_info(format!("Reusing Node.js {}", self.release.version));
        Ok(ExistingLayerStrategy::Keep)
    }
}
