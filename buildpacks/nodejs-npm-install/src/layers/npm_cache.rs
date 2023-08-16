use crate::NpmInstallBuildpack;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) struct NpmCacheLayer;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct NpmCacheLayerMetadata {
    layer_version: String,
}

const LAYER_VERSION: &str = "1";

impl Layer for NpmCacheLayer {
    type Buildpack = NpmInstallBuildpack;
    type Metadata = NpmCacheLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: false,
            cache: true,
        }
    }

    fn create(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        _layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        log_info("Creating new npm cache");
        LayerResultBuilder::new(NpmCacheLayerMetadata::default()).build()
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata.layer_version == LAYER_VERSION {
            log_info("Restoring npm cache");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            log_info("Npm cache has expired");
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl Default for NpmCacheLayerMetadata {
    fn default() -> Self {
        Self {
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}
