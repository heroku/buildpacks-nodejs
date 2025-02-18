use bullet_stream::state::Bullet;
use bullet_stream::Print;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Env;
use serde::{Deserialize, Serialize};
use std::io::Stdout;

use heroku_nodejs_utils::package_json::PackageManager;
use heroku_nodejs_utils::vrs::Version;

use crate::{cmd, CorepackBuildpack, CorepackBuildpackError};

pub(crate) fn enable_corepack(
    context: &BuildContext<CorepackBuildpack>,
    corepack_version: &Version,
    package_manager: &PackageManager,
    env: &Env,
    log: Print<Bullet<Stdout>>,
) -> Result<Print<Bullet<Stdout>>, libcnb::Error<CorepackBuildpackError>> {
    let new_metadata = ShimLayerMetadata {
        corepack_version: corepack_version.clone(),
        layer_version: LAYER_VERSION.to_string(),
    };

    let mut log = log.bullet("Enabling");

    let shim_layer = context.cached_layer(
        layer_name!("shim"),
        CachedLayerDefinition {
            launch: true,
            build: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &ShimLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match shim_layer.state {
        LayerState::Restored { .. } => {
            log = log.sub_bullet("Restoring Corepack shims");
        }
        LayerState::Empty { cause } => {
            match cause {
                EmptyLayerCause::NewlyCreated => {
                    log = log.sub_bullet("Creating Corepack shims");
                }
                _ => log = log.sub_bullet("Recreating Corepack shims (Corepack version changed)"),
            }
            shim_layer.write_metadata(new_metadata)?;
            std::fs::create_dir(shim_layer.path().join("bin"))
                .map_err(CorepackBuildpackError::ShimLayer)?;
        }
    }

    let log = cmd::corepack_enable(
        &package_manager.name,
        &shim_layer.path().join("bin"),
        env,
        log,
    )
    .map_err(CorepackBuildpackError::CorepackEnable)?;

    Ok(log.done())
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShimLayerMetadata {
    corepack_version: Version,
    layer_version: String,
}

const LAYER_VERSION: &str = "1";
