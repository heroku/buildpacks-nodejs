use crate::yarn::Yarn;
use crate::{NodeJsYarnBuildpack, NodeJsYarnBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::buildpack::StackId;
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
/// included in the source code.
pub(crate) struct DepsLayer {
    pub(crate) yarn: Yarn,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct DepsLayerMetadata {
    yarn: String,
    layer_version: String,
    stack_id: StackId,
}

#[derive(Error, Debug)]
#[error("Couldn't create yarn dependency cache: {0}")]
pub(crate) struct DepsLayerError(std::io::Error);

const LAYER_VERSION: &str = "1";

impl Layer for DepsLayer {
    type Buildpack = NodeJsYarnBuildpack;
    type Metadata = DepsLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: false,
            launch: false,
            cache: true,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsYarnBuildpackError> {
        fs::create_dir(layer_path.join("cache")).map_err(DepsLayerError)?;
        LayerResultBuilder::new(DepsLayerMetadata::new(self, context)).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data
            .content_metadata
            .metadata
            .is_cacheable(self, context)
        {
            log_info("Restoring yarn dependency cache");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            log_info("Clearing yarn dependency cache");
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl DepsLayerMetadata {
    fn is_cacheable(&self, layer: &DepsLayer, context: &BuildContext<NodeJsYarnBuildpack>) -> bool {
        self.yarn == layer.yarn.to_string()
            && self.stack_id == context.stack_id
            && self.layer_version == *LAYER_VERSION
    }

    fn new(layer: &DepsLayer, context: &BuildContext<NodeJsYarnBuildpack>) -> Self {
        DepsLayerMetadata {
            yarn: layer.yarn.to_string(),
            stack_id: context.stack_id.clone(),
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}
