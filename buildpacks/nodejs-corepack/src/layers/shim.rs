use heroku_nodejs_utils::vrs::Version;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::{CorepackBuildpack, CorepackBuildpackError};

/// `ShimLayer` is a layer for caching shims installed by corepack. These
/// shims will be cached until the corepack version changes.
pub(crate) struct ShimLayer {
    pub(crate) corepack_version: Version,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct ShimLayerMetadata {
    corepack_version: Version,
    layer_version: String,
}

#[derive(Error, Debug)]
#[error("Couldn't create corepack shim cache: {0}")]
pub(crate) struct ShimLayerError(std::io::Error);

const LAYER_VERSION: &str = "1";
const BIN_DIR: &str = "bin";

impl Layer for ShimLayer {
    type Buildpack = CorepackBuildpack;
    type Metadata = ShimLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn create(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, CorepackBuildpackError> {
        fs::create_dir(layer_path.join(BIN_DIR)).map_err(ShimLayerError)?;
        LayerResultBuilder::new(ShimLayerMetadata::new(self)).build()
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata.is_reusable(self) {
            log_info("Restoring corepack shim cache");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            log_info("Corepack change detected. Clearing corepack shim cache");
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl ShimLayerMetadata {
    fn is_reusable(&self, new_layer: &ShimLayer) -> bool {
        self.corepack_version == new_layer.corepack_version && self.layer_version == *LAYER_VERSION
    }

    fn new(layer: &ShimLayer) -> Self {
        ShimLayerMetadata {
            corepack_version: layer.corepack_version.clone(),
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}
