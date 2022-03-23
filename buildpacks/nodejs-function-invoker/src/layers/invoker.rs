use crate::{NodeJsInvokerBuildpack, NodeJsInvokerBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::buildpack::StackId;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::{decompress_tarball, download_file, log_info, move_directory_contents};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use tempfile::NamedTempFile;
use thiserror::Error;

/// A layer that installs the Node.js Invoker/Runtime package
pub struct InvokerLayer {
    pub package: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct InvokerLayerMetadata {
    layer_version: String,
    package: String,
    stack_id: StackId,
}

#[derive(Error, Debug)]
pub enum InvokerLayerError {
    #[error("Couldn't install Invoker with npm: #{0}")]
    NpmInstallError(std::io::Error),
}

const LAYER_VERSION: &str = "1";

impl Layer for InvokerLayer {
    type Buildpack = NodeJsInvokerBuildpack;
    type Metadata = InvokerLayerMetadata;

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
            "Installing Node.js Function Invoker {}",
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
            .map_err(InvokerLayerError::NpmInstallError)?;

        LayerResultBuilder::new(InvokerLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata == InvokerLayerMetadata::current(self, context) {
            log_info(format!("Reusing Node.js Function Invoker {}", self.package));
            Ok(ExistingLayerStrategy::Keep)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl InvokerLayerMetadata {
    fn current(layer: &InvokerLayer, context: &BuildContext<NodeJsInvokerBuildpack>) -> Self {
        InvokerLayerMetadata {
            package: layer.package.clone(),
            stack_id: context.stack_id.clone(),
            layer_version: String::from(LAYER_VERSION),
        }
    }
}
