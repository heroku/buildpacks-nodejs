use crate::{YarnBuildpack, YarnBuildpackError};
use heroku_nodejs_utils::inv::Release;
use libcnb::build::BuildContext;
use libcnb::data::buildpack::StackId;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::download::{download_file, DownloadError};
use libherokubuildpack::fs::move_directory_contents;
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tempfile::NamedTempFile;
use thiserror::Error;

/// A layer that downloads and installs the yarn cli
pub(crate) struct DistLayer {
    pub release: Release,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct DistLayerMetadata {
    layer_version: String,
    yarn_version: String,
    stack_id: StackId,
}

#[derive(Error, Debug)]
pub(crate) enum DistLayerError {
    #[error("Couldn't create tempfile for yarn CLI: {0}")]
    TempFile(std::io::Error),
    #[error("Couldn't download yarn CLI: {0}")]
    Download(DownloadError),
    #[error("Couldn't decompress yarn CLI: {0}")]
    Untar(std::io::Error),
    #[error("Couldn't move yarn CLI to the target location: {0}")]
    Installation(std::io::Error),
}

const LAYER_VERSION: &str = "1";

impl Layer for DistLayer {
    type Buildpack = YarnBuildpack;
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
    ) -> Result<LayerResult<Self::Metadata>, YarnBuildpackError> {
        let yarn_tgz = NamedTempFile::new().map_err(DistLayerError::TempFile)?;

        log_info(format!("Downloading yarn {}", self.release.version));
        download_file(&self.release.url, yarn_tgz.path()).map_err(DistLayerError::Download)?;

        log_info(format!("Extracting yarn {}", self.release.version));
        decompress_tarball(&mut yarn_tgz.into_file(), &layer_path)
            .map_err(DistLayerError::Untar)?;

        log_info(format!("Installing yarn {}", self.release.version));
        let dist_name = format!("yarn-v{}", self.release.version);
        let dist_path = Path::new(layer_path).join(dist_name);
        move_directory_contents(dist_path, layer_path).map_err(DistLayerError::Installation)?;

        LayerResultBuilder::new(DistLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata == DistLayerMetadata::current(self, context) {
            log_info(format!("Reusing yarn {}", self.release.version));
            Ok(ExistingLayerStrategy::Keep)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl DistLayerMetadata {
    fn current(layer: &DistLayer, context: &BuildContext<YarnBuildpack>) -> Self {
        DistLayerMetadata {
            yarn_version: layer.release.version.to_string(),
            stack_id: context.stack_id.clone(),
            layer_version: String::from(LAYER_VERSION),
        }
    }
}
