use std::path::Path;

use crate::util;

use tempfile::NamedTempFile;

use crate::{NodejsRuntimeBuildpack, NodejsBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb_nodejs::{package_json, versions};

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
        println!("---> Checking Node.js version");

        let inventory_path = context.buildpack_dir.join("inventory.toml");
        let inventory = std::fs::read_to_string(inventory_path).map_err(NodejsBuildpackError::InventoryReadError)?;
        let software: versions::Software = toml::from_str(&inventory).map_err(NodejsBuildpackError::InventoryParseError)?;

        let pjson_path = context.app_dir.join("package.json");
        let pjson = package_json::read(pjson_path).map_err(NodejsBuildpackError::PackageJsonError)?;
        let node_version_range = match pjson.engines {
            None => versions::Req::any(),
            Some(engines) => match engines.node {
                None => versions::Req::any(),
                Some(v) => v
            }
        };
        let target_node_release = software.resolve(node_version_range, "x64", "linux").ok_or(NodejsBuildpackError::UnknownVersionError())?;

        println!("---> Downloading and Installing Node.js {}", target_node_release.version);

        let node_tgz = NamedTempFile::new().map_err(NodejsBuildpackError::CreateTempFileError)?;

        util::download(
            &context.buildpack_descriptor.metadata.nodejs_runtime_url,
            node_tgz.path(),
        )
        .map_err(NodejsBuildpackError::NodejsDownloadError)?;

        util::untar(node_tgz.path(), &layer_path).map_err(NodejsBuildpackError::NodejsUntarError)?;

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
