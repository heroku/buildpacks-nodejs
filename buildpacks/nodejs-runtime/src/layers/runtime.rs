use std::path::Path;

use crate::util;

use tempfile::NamedTempFile;

use crate::{NodejsRuntimeBuildpack, NodejsBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};

pub struct RuntimeLayer;

impl Layer for RuntimeLayer {
    type Buildpack = NodejsRuntimeBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: false,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodejsBuildpackError> {
        println!("---> Download and extracting Node.js");

        let ruby_tgz =
            NamedTempFile::new().map_err(NodejsBuildpackError::CouldNotCreateTemporaryFile)?;

        util::download(
            &context.buildpack_descriptor.metadata.nodejs_runtime_url,
            ruby_tgz.path(),
        )
        .map_err(NodejsBuildpackError::NodejsDownloadError)?;

        util::untar(ruby_tgz.path(), &layer_path).map_err(NodejsBuildpackError::NodejsUntarError)?;

        LayerResultBuilder::new(GenericMetadata::default())
            .env(
                LayerEnv::new()
                    .chainable_insert(
                        Scope::All,
                        ModificationBehavior::Prepend,
                        "PATH",
                        context.app_dir.join("/where/is/node"),
                    )
            )
            .build()
    }
}
