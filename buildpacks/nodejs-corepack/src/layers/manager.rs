use heroku_nodejs_utils::package_json::PackageManager;
use heroku_nodejs_utils::vrs::Version;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb::Buildpack;
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;

use crate::{CorepackBuildpack, CorepackBuildpackError};

/// `ManagerLayer` is a layer for caching shims installed by corepack. These
/// shims will be cached until the corepack version changes.
pub(crate) struct ManagerLayer {
    pub(crate) package_manager: PackageManager,
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct ManagerLayerMetadata {
    manager_name: String,
    manager_version: Version,
    layer_version: String,
}

#[derive(Error, Debug)]
#[error("Couldn't create corepack package manager cache: {0}")]
pub(crate) struct ManagerLayerError(std::io::Error);

const LAYER_VERSION: &str = "1";
const CACHE_DIR: &str = "cache";

impl Layer for ManagerLayer {
    type Buildpack = CorepackBuildpack;
    type Metadata = ManagerLayerMetadata;

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
    ) -> Result<LayerResult<Self::Metadata>, CorepackBuildpackError> {
        let cache_path = layer_path.join(CACHE_DIR);
        fs::create_dir(&cache_path).map_err(ManagerLayerError)?;
        LayerResultBuilder::new(ManagerLayerMetadata::new(self))
            .env(LayerEnv::new().chainable_insert(
                Scope::All,
                ModificationBehavior::Override,
                "COREPACK_HOME",
                cache_path,
            ))
            .build()
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata.is_reusable(self) {
            log_info("Restoring corepack package manager cache");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            log_info("Package manager change detected. Clearing corepack package manager cache");
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl ManagerLayerMetadata {
    fn is_reusable(&self, new_layer: &ManagerLayer) -> bool {
        self.manager_name == new_layer.package_manager.name
            && self.manager_version == new_layer.package_manager.version
            && self.layer_version == *LAYER_VERSION
    }

    fn new(layer: &ManagerLayer) -> Self {
        ManagerLayerMetadata {
            manager_name: layer.package_manager.name.clone(),
            manager_version: layer.package_manager.version.clone(),
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}
