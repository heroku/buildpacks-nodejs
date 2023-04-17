use crate::{PnpmInstallBuildpack, PnpmInstallBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libherokubuildpack::log::log_info;
use std::path::Path;

/// `VirtualStoreLayer` is a layer for the pnpm virtual store. It's contents
/// consist of hardlinks to a subset of modules in the `AddressableStoreLayer`.
/// It is not cached, because it's contents are hardlinks to content cached
/// in `AddressableStoreLayer`.
pub(crate) struct VirtualStoreLayer;

impl Layer for VirtualStoreLayer {
    type Buildpack = PnpmInstallBuildpack;
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
        _context: &BuildContext<Self::Buildpack>,
        _layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, PnpmInstallBuildpackError> {
        log_info("Creating pnpm virtual store");
        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}
