use crate::NpmInstallBuildpack;
use commons::output::section_log::{log_step, SectionLogger};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub(crate) struct NpmCacheLayer<'a> {
    // this ensures we have a logging section already created
    pub(crate) _section_logger: &'a dyn SectionLogger,
}

impl<'a> Layer for NpmCacheLayer<'a> {
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
        log_step("Creating npm cache");
        LayerResultBuilder::new(NpmCacheLayerMetadata::default()).build()
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata.layer_version == LAYER_VERSION {
            log_step("Restoring npm cache");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            log_step("Recreating npm cache (layer version changed)");
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct NpmCacheLayerMetadata {
    layer_version: String,
}

impl Default for NpmCacheLayerMetadata {
    fn default() -> Self {
        Self {
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}
