use crate::{PnpmInstallBuildpack, PnpmInstallBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// `AddressableStoreLayer` is a layer for the pnpm global, addressable module
/// store. It's available at build and is cached. It's not available at runtime,
/// as the subset of modules in use are hardlinked in the `VirtualStoreLayer`.
pub(crate) struct AddressableStoreLayer;

#[derive(Deserialize, Serialize, Clone, PartialEq)]
pub(crate) struct AddressableStoreLayerMetadata {
    layer_version: String,
}

const LAYER_VERSION: &str = "1";

impl Layer for AddressableStoreLayer {
    type Buildpack = PnpmInstallBuildpack;
    type Metadata = AddressableStoreLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: false,
            cache: true,
        }
    }

    fn create(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        _layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, PnpmInstallBuildpackError> {
        log_info("Creating new pnpm content-addressable store");
        LayerResultBuilder::new(AddressableStoreLayerMetadata::default()).build()
    }

    fn existing_layer_strategy(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata.layer_version == LAYER_VERSION {
            log_info("Restoring pnpm content-addressable store from cache");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            log_info("Cached pnpm content-addressable store has expired");
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl Default for AddressableStoreLayerMetadata {
    fn default() -> Self {
        Self {
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}
