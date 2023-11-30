use crate::{PnpmInstallBuildpack, PnpmInstallBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libherokubuildpack::log::log_info;
use std::fs::create_dir;
use std::os::unix::fs::symlink;
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
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, PnpmInstallBuildpackError> {
        log_info("Creating pnpm virtual store");

        // Create a directory for dependencies in the virtual store.
        create_dir(layer_path.join("store")).map_err(PnpmInstallBuildpackError::VirtualLayer)?;

        // Install a symlink from {virtual_layer}/node_modules to
        // {app_dir}/node_modules, so that dependencies in
        // {virtual_layer}/store/ can find their dependencies via the Node
        // module loader's ancestor directory traversal.
        symlink(
            context.app_dir.join("node_modules"),
            layer_path.join("node_modules"),
        )
        .map_err(PnpmInstallBuildpackError::VirtualLayer)?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}
