use crate::yarn::Yarn;
use crate::{YarnBuildpack, YarnBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// `DepsLayer` is a layer for caching yarn dependencies from build to build.
/// This layer is irrelevant in zero-install mode, as cached dependencies are
/// included in the source code. Yarn only prunes unused dependencies in a few
/// scenarios, so the cache is invalidated after a number of builds to prevent
/// unbound cache growth.
pub(crate) struct DepsLayer {
    pub(crate) yarn: Yarn,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct DepsLayerMetadata {
    // Using float here due to [an issue with lifecycle's handling of integers](https://github.com/buildpacks/lifecycle/issues/884)
    cache_usage_count: f32,
    layer_version: String,
    yarn: Yarn,
}

#[derive(Error, Debug)]
#[error("Couldn't create yarn dependency cache: {0}")]
pub(crate) struct DepsLayerError(std::io::Error);

const LAYER_VERSION: &str = "1";
const MAX_CACHE_USAGE_COUNT: f32 = 150.0;
const CACHE_DIR: &str = "cache";

impl Layer for DepsLayer {
    type Buildpack = YarnBuildpack;
    type Metadata = DepsLayerMetadata;

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
    ) -> Result<LayerResult<Self::Metadata>, YarnBuildpackError> {
        fs::create_dir(layer_path.join(CACHE_DIR)).map_err(DepsLayerError)?;
        LayerResultBuilder::new(DepsLayerMetadata::new(self)).build()
    }

    fn update(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer: &LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, YarnBuildpackError> {
        LayerResultBuilder::new(DepsLayerMetadata {
            cache_usage_count: layer.content_metadata.metadata.cache_usage_count + 1.0,
            ..DepsLayerMetadata::new(self)
        })
        .build()
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata.is_reusable(self) {
            log_info("Restoring yarn dependency cache");
            Ok(ExistingLayerStrategy::Update)
        } else {
            log_info("Clearing yarn dependency cache");
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl DepsLayerMetadata {
    fn is_reusable(&self, new_layer: &DepsLayer) -> bool {
        self.yarn == new_layer.yarn
            && self.layer_version == *LAYER_VERSION
            && self.cache_usage_count < MAX_CACHE_USAGE_COUNT
    }

    fn new(layer: &DepsLayer) -> Self {
        DepsLayerMetadata {
            yarn: layer.yarn.clone(),
            layer_version: LAYER_VERSION.to_string(),
            cache_usage_count: 1.0,
        }
    }
}
