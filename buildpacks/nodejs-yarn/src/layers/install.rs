use crate::{NodeJsYarnBuildpack, NodeJsYarnBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::buildpack::StackId;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::{Buildpack, Env};
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// InstallLayer is a layer that runs `yarn install` and maintains the yarn
/// cache.
pub struct InstallLayer {
    pub yarn_env: Env,
    pub yarn_app_cache: bool,
    pub yarn_major_version: usize,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct InstallLayerMetadata {
    yarn_app_cache: bool,
    yarn_major_version: usize,
    layer_version: usize,
    stack_id: StackId,
}

#[derive(Error, Debug)]
pub enum InstallLayerError {
    #[error("Couldn't execute `yarn`: {0}")]
    YarnCommand(std::io::Error),
    #[error("Couldn't install packages with `yarn install`: {0}")]
    YarnInstall(std::process::ExitStatus),
}

const LAYER_VERSION: usize = 1 as usize;

impl Layer for InstallLayer {
    type Buildpack = NodeJsYarnBuildpack;
    type Metadata = InstallLayerMetadata;

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
        self.install(layer_path)?;
        LayerResultBuilder::new(InstallLayerMetadata::current(self, context)).build()
    }

    fn update(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer: &LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsYarnBuildpackError> {
        self.install(&layer.path)?;
        LayerResultBuilder::new(InstallLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if !self.yarn_app_cache
            && layer_data.content_metadata.metadata == InstallLayerMetadata::current(self, context)
        {
            log_info("Restoring yarn cache");
            Ok(ExistingLayerStrategy::Update)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}
impl InstallLayer {
    fn install(&self, layer_path: &Path) -> Result<(), InstallLayerError> {
        log_info("Running yarn install");

        let mut args = vec!["install", "--frozen-lockfile"];
        let path = layer_path.to_string_lossy().to_owned();
        if !self.yarn_app_cache {
            args.append(&mut vec!["--cache-folder", &path]);
        }
        if self.yarn_major_version < 1 {
            args.append(&mut vec!["--production", "false"]);
        }

        let output = Command::new("yarn")
            .args(args)
            .envs(&self.yarn_env)
            .output()
            .map_err(InstallLayerError::YarnCommand)?;

        output.status.success().then(|| ()).ok_or_else(|| {
            // log `yarn install` stderr and stdout *only* if it fails.
            io::stdout().write_all(&output.stdout).ok();
            io::stderr().write_all(&output.stderr).ok();
            InstallLayerError::YarnInstall(output.status)
        })
    }
}

impl InstallLayerMetadata {
    fn current(layer: &InstallLayer, context: &BuildContext<NodeJsYarnBuildpack>) -> Self {
        InstallLayerMetadata {
            yarn_app_cache: layer.yarn_app_cache,
            yarn_major_version: layer.yarn_major_version,
            stack_id: context.stack_id.clone(),
            layer_version: LAYER_VERSION,
        }
    }
}
