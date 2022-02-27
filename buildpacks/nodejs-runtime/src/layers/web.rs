#![warn(clippy::pedantic)]
use std::path::Path;

use crate::{NodeJsRuntimeBuildpack, NodeJsBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::{additional_buildpack_binary_path};


/// A layer that sets WEB_MEMORY and WEB_CONCURRENCY via exec.d
pub struct WebLayer;

impl Layer for WebLayer {
    type Buildpack = NodeJsRuntimeBuildpack;
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
    ) -> Result<LayerResult<Self::Metadata>, NodeJsBuildpackError> {
        LayerResultBuilder::new(GenericMetadata::default())
            .exec_d_program(
                "web_env",
                additional_buildpack_binary_path!("web_env"),
            )
            .build()
    }
}
