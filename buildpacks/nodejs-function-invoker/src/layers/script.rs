use crate::NodeJsInvokerBuildpack;
use crate::NodeJsInvokerBuildpackError;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
#[allow(deprecated)]
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub(crate) const NODEJS_RUNTIME_SCRIPT: &str = "nodejs-runtime.sh";

/// A layer that installs a bash script used for passing environment variables
/// on to sf-fx-runtime-nodejs as arguments.
pub(crate) struct ScriptLayer;

#[allow(deprecated)]
impl Layer for ScriptLayer {
    type Buildpack = NodeJsInvokerBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            launch: true,
            build: false,
            cache: false,
        }
    }

    fn create(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsInvokerBuildpackError> {
        let layer_bin_dir = layer_path.join("bin");
        let destination = layer_bin_dir.join(NODEJS_RUNTIME_SCRIPT);

        fs::create_dir_all(&layer_bin_dir).map_err(ScriptLayerError::CouldNotWriteRuntimeScript)?;

        fs::write(&destination, include_bytes!("../../opt/nodejs-runtime.sh"))
            .map_err(ScriptLayerError::CouldNotWriteRuntimeScript)?;

        fs::set_permissions(&destination, fs::Permissions::from_mode(0o755))
            .map_err(ScriptLayerError::CouldNotSetExecutableBitForRuntimeScript)?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum ScriptLayerError {
    #[error("Could not write runtime script to layer: {0}")]
    CouldNotWriteRuntimeScript(std::io::Error),
    #[error("Could not set executable bit on runtime script: {0}")]
    CouldNotSetExecutableBitForRuntimeScript(std::io::Error),
}
