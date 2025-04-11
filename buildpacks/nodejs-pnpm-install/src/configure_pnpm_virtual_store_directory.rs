use bullet_stream::state::SubBullet;
use bullet_stream::Print;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::UncachedLayerDefinition;
use libcnb::Env;
use std::fs::create_dir;
use std::io::Stderr;
use std::os::unix::fs::symlink;

use crate::{cmd, PnpmInstallBuildpack, PnpmInstallBuildpackError};

pub(crate) fn configure_pnpm_virtual_store_directory(
    context: &BuildContext<PnpmInstallBuildpack>,
    env: &Env,
    mut log: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, libcnb::Error<PnpmInstallBuildpackError>> {
    let virtual_layer = context.uncached_layer(
        layer_name!("virtual"),
        UncachedLayerDefinition {
            build: true,
            launch: true,
        },
    )?;

    log = log.sub_bullet("Creating pnpm virtual store");
    let virtual_store_dir = virtual_layer.path().join("store");
    // Create a directory for dependencies in the virtual store.
    create_dir(&virtual_store_dir)
        .map_err(|e| PnpmInstallBuildpackError::CreateDirectory(virtual_store_dir.clone(), e))?;

    cmd::pnpm_set_virtual_dir(env, &virtual_store_dir)
        .map_err(PnpmInstallBuildpackError::PnpmSetVirtualStoreDir)?;

    // Install a symlink from {virtual_layer}/node_modules to
    // {app_dir}/node_modules, so that dependencies in
    // {virtual_layer}/store/ can find their dependencies via the Node
    // module loader's ancestor directory traversal.
    let from = context.app_dir.join("node_modules");
    let to = virtual_layer.path().join("node_modules");
    symlink(&from, &to).map_err(|source| PnpmInstallBuildpackError::CreateSymlink {
        from,
        to,
        source,
    })?;

    Ok(log)
}
