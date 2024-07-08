use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Env;
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};

use heroku_nodejs_utils::package_json::PackageManager;
use heroku_nodejs_utils::vrs::Version;

use crate::{cmd, CorepackBuildpack, CorepackBuildpackError};

pub(crate) fn enable_corepack(
    context: &BuildContext<CorepackBuildpack>,
    corepack_version: &Version,
    package_manager: &PackageManager,
    env: &Env,
) -> Result<(), libcnb::Error<CorepackBuildpackError>> {
    let new_metadata = ShimLayerMetadata {
        corepack_version: corepack_version.clone(),
        layer_version: LAYER_VERSION.to_string(),
    };

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
            log_info("Restoring corepack shim cache");
        }
        LayerState::Empty { .. } => {
            log_info("Corepack change detected. Clearing corepack shim cache");
            shim_layer.write_metadata(new_metadata)?;
            std::fs::create_dir(shim_layer.path().join("bin"))
                .map_err(CorepackBuildpackError::ShimLayer)?;
        }
    }

    cmd::corepack_enable(&package_manager.name, &shim_layer.path().join("bin"), env)
        .map_err(CorepackBuildpackError::CorepackEnable)?;

    Ok(())
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShimLayerMetadata {
    corepack_version: Version,
    layer_version: String,
}

const LAYER_VERSION: &str = "1";
