use std::os::unix::fs::PermissionsExt;

use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::UncachedLayerDefinition;

use crate::{NodeJsInvokerBuildpack, NodeJsInvokerBuildpackError};

/// Attaches a bash script used for passing environment variables
/// on to sf-fx-runtime-nodejs as arguments.
pub(crate) fn attach_startup_script(
    context: &BuildContext<NodeJsInvokerBuildpack>,
) -> Result<(), libcnb::Error<NodeJsInvokerBuildpackError>> {
    let script_layer = context.uncached_layer(
        layer_name!("script"),
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;

    let layer_bin_dir = script_layer.path().join("bin");
    let destination = layer_bin_dir.join(NODEJS_RUNTIME_SCRIPT);

    std::fs::create_dir_all(&layer_bin_dir)
        .map_err(ScriptLayerError::CouldNotWriteRuntimeScript)?;

    std::fs::write(&destination, include_bytes!("../opt/nodejs-runtime.sh"))
        .map_err(ScriptLayerError::CouldNotWriteRuntimeScript)?;

    std::fs::set_permissions(&destination, std::fs::Permissions::from_mode(0o755))
        .map_err(ScriptLayerError::CouldNotSetExecutableBitForRuntimeScript)?;

    Ok(())
}

pub(crate) const NODEJS_RUNTIME_SCRIPT: &str = "nodejs-runtime.sh";

#[derive(thiserror::Error, Debug)]
pub(crate) enum ScriptLayerError {
    #[error("Could not write runtime script to layer: {0}")]
    CouldNotWriteRuntimeScript(std::io::Error),
    #[error("Could not set executable bit on runtime script: {0}")]
    CouldNotSetExecutableBitForRuntimeScript(std::io::Error),
}

impl From<ScriptLayerError> for libcnb::Error<NodeJsInvokerBuildpackError> {
    fn from(value: ScriptLayerError) -> Self {
        libcnb::Error::BuildpackError(NodeJsInvokerBuildpackError::ScriptLayer(value))
    }
}
