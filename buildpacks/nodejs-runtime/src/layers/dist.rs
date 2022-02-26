use std::path::Path;

use crate::util;

use tempfile::NamedTempFile;
use std::ffi::OsString;

use crate::{NodeJsRuntimeBuildpack, NodeJsBuildpackError};
use libcnb::build::BuildContext;
use libcnb::Buildpack;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerData, LayerResultBuilder, ExistingLayerStrategy};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb_nodejs::versions::{Release};

/// A layer that downloads the Node.js distribution artifacts
pub struct DistLayer {
    pub release: Release
}

impl Layer for DistLayer {
    type Buildpack = NodeJsRuntimeBuildpack;
    type Metadata = GenericMetadata;

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
    ) -> Result<LayerResult<Self::Metadata>, NodeJsBuildpackError> {
        println!("---> Downloading and Installing Node.js {}", self.release.version);
        let node_tgz = NamedTempFile::new().map_err(NodeJsBuildpackError::CreateTempFileError)?;
        util::download(
            self.release.url.clone(),
            node_tgz.path(),
        )
        .map_err(NodeJsBuildpackError::DownloadError)?;

        util::untar(node_tgz.path(), &layer_path).map_err(NodeJsBuildpackError::UntarError)?;

        let dist_name = format!("node-v{}-{}", self.release.version.to_string(), "linux-x64");
        let bin_path = Path::new(layer_path).join(dist_name).join("bin");
        if !bin_path.exists() {
            return Err(NodeJsBuildpackError::ReadBinError())
        }

        LayerResultBuilder::new(GenericMetadata::default())
            .env(
                LayerEnv::new()
                    .chainable_insert(
                        Scope::All,
                        ModificationBehavior::Prepend,
                        "PATH",
                        bin_path
                    )
                    .chainable_insert(
                        Scope::All,
                        ModificationBehavior::Delimiter,
                        "PATH",
                        ":"
                    )
                    .chainable_insert(
                        Scope::Build,
                        ModificationBehavior::Override,
                        "HEROKU_NODEJS_VERSION",
                        self.release.version.to_string()
                    )
            )
            .build()
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        let new_version: OsString = self.release.version.to_string().into();
        let old_version = layer_data.env
            .apply_to_empty(Scope::All)
            .get("HEROKU_NODEJS_VERSION");

        match old_version {
            Some(ov) if ov == new_version => {
                println!("---> Reusing Node.js {}", self.release.version.to_string());
                return Ok(ExistingLayerStrategy::Keep);
            },
            _ => Ok(ExistingLayerStrategy::Recreate)
        }
    }
}
