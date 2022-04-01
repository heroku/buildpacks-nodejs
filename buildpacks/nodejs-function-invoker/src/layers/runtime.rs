use crate::{NodeJsInvokerBuildpack, NodeJsInvokerBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::buildpack::StackId;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::log_info;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// A layer that installs the Node.js Invoker/Runtime package
pub struct RuntimeLayer {
    pub package: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct RuntimeLayerMetadata {
    layer_version: String,
    package: String,
    stack_id: StackId,
}

#[derive(Error, Debug)]
pub enum RuntimeLayerError {
    #[error("Couldn't run `npm install` command: {0}")]
    NpmCommandError(std::io::Error),
    #[error("Couldn't install invoker runtime with `npm install`: #{0}")]
    NpmInstallError(std::process::ExitStatus),
}

const LAYER_VERSION: &str = "1";

impl Layer for RuntimeLayer {
    type Buildpack = NodeJsInvokerBuildpack;
    type Metadata = RuntimeLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsInvokerBuildpackError> {
        log_info(format!(
            "Installing Node.js Function Invoker Runtime {}",
            self.package
        ));

        Command::new("npm")
            .args([
                "install",
                "-g",
                "--prefix",
                &layer_path.to_string_lossy(),
                &self.package,
            ])
            .output()
            .map_err(RuntimeLayerError::NpmCommandError)
            .and_then(|output| {
                output.status.success().then(|| ()).ok_or_else(|| {
                    // log `npm install` stderr and stdout *only* if it fails.
                    io::stdout().write_all(&output.stdout).ok();
                    io::stderr().write_all(&output.stderr).ok();
                    RuntimeLayerError::NpmInstallError(output.status)
                })
            })?;

        LayerResultBuilder::new(RuntimeLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata == RuntimeLayerMetadata::current(self, context) {
            log_info(format!(
                "Reusing Node.js Function Invoker Runtime {}",
                self.package
            ));
            Ok(ExistingLayerStrategy::Keep)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl RuntimeLayerMetadata {
    fn current(layer: &RuntimeLayer, context: &BuildContext<NodeJsInvokerBuildpack>) -> Self {
        RuntimeLayerMetadata {
            package: layer.package.clone(),
            stack_id: context.stack_id.clone(),
            layer_version: String::from(LAYER_VERSION),
        }
    }
}
