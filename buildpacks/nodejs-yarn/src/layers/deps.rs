use crate::{NodeJsYarnBuildpack, NodeJsYarnBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::buildpack::StackId;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::{Buildpack, Env};
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// `DepsLayer` is a layer that uses `yarn install` to cache and install
/// application dependencies.
pub struct DepsLayer {
    pub yarn_env: Env,
    pub yarn_app_cache: bool,
    pub yarn_major_version: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct DepsLayerMetadata {
    yarn_app_cache: bool,
    yarn_major_version: String,
    layer_version: String,
    stack_id: StackId,
}

#[derive(Error, Debug)]
pub enum DepsLayerError {
    #[error("Couldn't create yarn cache folder: {0}")]
    CacheFolder(std::io::Error),
    #[error("Couldn't execute `yarn`: {0}")]
    YarnCommand(std::io::Error),
    #[error("Couldn't install packages with `yarn install`: {0}")]
    YarnInstall(std::process::ExitStatus),
}

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
        if !self.yarn_app_cache {
            fs::create_dir(layer_path.join("cache")).map_err(DepsLayerError::CacheFolder)?;
        }
        self.install(layer_path)?;
        LayerResultBuilder::new(DepsLayerMetadata::current(self, context)).build()
    }

    fn update(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer: &LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsYarnBuildpackError> {
        self.install(&layer.path)?;
        LayerResultBuilder::new(DepsLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if !self.yarn_app_cache
            && layer_data.content_metadata.metadata == DepsLayerMetadata::current(self, context)
        {
            log_info("Restoring yarn dependency cache");
            Ok(ExistingLayerStrategy::Update)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl DepsLayer {
    fn install(&self, layer_path: &Path) -> Result<(), DepsLayerError> {
        let mut args = vec!["install"];
        let cache_path = layer_path.join("cache");
        let cache_folder = cache_path.to_string_lossy().clone();
        if self.yarn_major_version == "1" {
            args.append(&mut vec![
                "--cache-folder",
                &cache_folder,
                "--frozen-lockfile",
            ]);
        } else {
            args.append(&mut vec!["--immutable", "--immutable-cache"]);
        }

        let mut process = Command::new("yarn")
            .args(args)
            .envs(&self.yarn_env)
            .spawn()
            .map_err(DepsLayerError::YarnCommand)?;

        let status = process.wait().map_err(DepsLayerError::YarnCommand)?;

        status
            .success()
            .then_some(())
            .ok_or(DepsLayerError::YarnInstall(status))
    }
}

impl DepsLayerMetadata {
    fn current(layer: &DepsLayer, context: &BuildContext<NodeJsYarnBuildpack>) -> Self {
        DepsLayerMetadata {
            yarn_app_cache: layer.yarn_app_cache,
            yarn_major_version: layer.yarn_major_version.clone(),
            stack_id: context.stack_id.clone(),
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}
