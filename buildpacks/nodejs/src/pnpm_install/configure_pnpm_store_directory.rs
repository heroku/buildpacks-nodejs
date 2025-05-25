use crate::pnpm_install::cmd;
use crate::pnpm_install::main::PnpmInstallBuildpackError;
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Env;
use serde::{Deserialize, Serialize};

pub(crate) fn configure_pnpm_store_directory(
    context: &BuildContext<NodeJsBuildpack>,
    env: &Env,
) -> Result<(), libcnb::Error<NodeJsBuildpackError>> {
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
            print::sub_bullet("Restoring pnpm content-addressable store from cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                print::sub_bullet("Cached pnpm content-addressable store has expired");
            }
            print::sub_bullet("Creating new pnpm content-addressable store");
            addressable_layer.write_metadata(new_metadata)?;
        }
    }

    cmd::pnpm_set_store_dir(env, &addressable_layer.path())
        .map_err(PnpmInstallBuildpackError::PnpmSetStoreDir)
        .map_err(Into::into)
}

const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AddressableStoreLayerMetadata {
    layer_version: String,
}
