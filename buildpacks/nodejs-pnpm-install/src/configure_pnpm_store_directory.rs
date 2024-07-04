use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Env;
use libherokubuildpack::log::log_info;
use serde::{Deserialize, Serialize};

use crate::{cmd, PnpmInstallBuildpack, PnpmInstallBuildpackError};

pub(crate) fn configure_pnpm_store_directory(
    context: &BuildContext<PnpmInstallBuildpack>,
    env: &Env,
) -> Result<(), libcnb::Error<PnpmInstallBuildpackError>> {
    let new_metadata = AddressableStoreLayerMetadata {
        layer_version: LAYER_VERSION.to_string(),
    };

    let addressable_layer = context.cached_layer(
        layer_name!("addressable"),
        CachedLayerDefinition {
            build: true,
            launch: false,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &AddressableStoreLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match addressable_layer.state {
        LayerState::Restored { .. } => {
            log_info("Restoring pnpm content-addressable store from cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                log_info("Cached pnpm content-addressable store has expired");
            }
            log_info("Creating new pnpm content-addressable store");
            addressable_layer.write_metadata(new_metadata)?;
        }
    }

    cmd::pnpm_set_store_dir(env, &addressable_layer.path())
        .map_err(PnpmInstallBuildpackError::PnpmDir)?;

    Ok(())
}

const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddressableStoreLayerMetadata {
    layer_version: String,
}
