use std::io::{stderr, stdout, Write};
use std::process::Command;

use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{NodeJsInvokerBuildpack, NodeJsInvokerBuildpackError};

pub(crate) fn install_nodejs_function_runtime(
    context: &BuildContext<NodeJsInvokerBuildpack>,
    package: &str,
) -> Result<(), libcnb::Error<NodeJsInvokerBuildpackError>> {
    let new_metadata = RuntimeLayerMetadata {
        package: package.to_string(),
        layer_version: LAYER_VERSION.to_string(),
        arch: context.target.arch.clone(),
        os: context.target.os.clone(),
    };

    let runtime_layer = context.cached_layer(
        layer_name!("runtime"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &RuntimeLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match runtime_layer.state {
        LayerState::Restored { .. } => {
            log_info(format!(
                "Reusing Node.js Function Invoker Runtime {package}",
            ));
        }
        LayerState::Empty { .. } => {
            runtime_layer.write_metadata(new_metadata)?;

            log_info(format!(
                "Installing Node.js Function Invoker Runtime {package}",
            ));

            Command::new("npm")
                .args([
                    "install",
                    "-g",
                    "--prefix",
                    &runtime_layer.path().to_string_lossy(),
                    package,
                ])
                .output()
                .map_err(RuntimeLayerError::NpmCommandError)
                .and_then(|output| {
                    output.status.success().then_some(()).ok_or_else(|| {
                        // log `npm install` stderr and stdout *only* if it fails.
                        stdout().write_all(&output.stdout).ok();
                        stderr().write_all(&output.stderr).ok();
                        RuntimeLayerError::NpmInstallError(output.status)
                    })
                })?;
        }
    }

    Ok(())
}

const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeLayerMetadata {
    layer_version: String,
    package: String,
    arch: String,
    os: String,
}

#[derive(Error, Debug)]
pub(crate) enum RuntimeLayerError {
    #[error("Couldn't run `npm install` command: {0}")]
    NpmCommandError(std::io::Error),
    #[error("Couldn't install invoker runtime with `npm install`: #{0}")]
    NpmInstallError(std::process::ExitStatus),
}

impl From<RuntimeLayerError> for libcnb::Error<NodeJsInvokerBuildpackError> {
    fn from(value: RuntimeLayerError) -> Self {
        libcnb::Error::BuildpackError(NodeJsInvokerBuildpackError::RuntimeLayer(value))
    }
}
