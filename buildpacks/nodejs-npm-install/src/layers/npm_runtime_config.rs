use crate::NpmInstallBuildpack;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::{additional_buildpack_binary_path, Buildpack};
use std::path::Path;

pub(crate) struct NpmRuntimeConfigLayer;

impl Layer for NpmRuntimeConfigLayer {
    type Buildpack = NpmInstallBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: false,
            launch: true,
            cache: false,
        }
    }

    fn create(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        _layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        LayerResultBuilder::new(GenericMetadata::default())
            .exec_d_program(
                "npm_runtime_config",
                additional_buildpack_binary_path!("npm_runtime_config"),
            )
            .build()
    }
}
