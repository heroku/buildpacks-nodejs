#![warn(clippy::pedantic)]
use std::path::Path;

use crate::{NodeJsEngineBuildpack, NodeJsEngineBuildpackError};
use libcnb::additional_buildpack_binary_path;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};

/// A layer that sets `WEB_MEMORY` and `WEB_CONCURRENCY` via exec.d
pub struct WebEnvLayer;

impl Layer for WebEnvLayer {
    type Buildpack = NodeJsEngineBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: false,
            launch: true,
            cache: false,
        }
    }

    fn create(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        _layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsEngineBuildpackError> {
        LayerResultBuilder::new(GenericMetadata::default())
            .exec_d_program("web_env", additional_buildpack_binary_path!("web_env"))
            .build()
    }
}
