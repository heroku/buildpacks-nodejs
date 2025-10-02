use super::cmd;
use super::main::CorepackBuildpackError;
use crate::utils::package_json::PackageManager;
use crate::utils::vrs::Version;
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::global::print;
use bullet_stream::style;
use libcnb::Env;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use serde::{Deserialize, Serialize};

pub(crate) fn prepare_corepack(
    context: &BuildpackBuildContext,
    package_manager: &PackageManager,
    mut env: Env,
) -> BuildpackResult<Env> {
    let new_metadata = ManagerLayerMetadata {
        manager_name: package_manager.name.clone(),
        manager_version: package_manager.version.clone(),
        layer_version: LAYER_VERSION.to_string(),
    };

    let manager_layer = context.cached_layer(
        layer_name!("mgr"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &ManagerLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    print::bullet(format!(
        "Installing {}",
        style::value(package_manager.to_string())
    ));

    match manager_layer.state {
        LayerState::Restored { .. } => {
            print::sub_bullet("Restoring Corepack package manager");
        }
        LayerState::Empty { cause } => {
            match cause {
                EmptyLayerCause::NewlyCreated => {
                    print::sub_bullet("Creating Corepack package manager");
                }
                _ => {
                    print::sub_bullet("Recreating Corepack package manager (version changed)");
                }
            }
            manager_layer.write_metadata(new_metadata)?;

            let cache_path = manager_layer.path().join("cache");
            std::fs::create_dir(&cache_path)
                .map_err(|e| CorepackBuildpackError::CreateCacheDirectory(cache_path.clone(), e))?;

            manager_layer.write_env(LayerEnv::new().chainable_insert(
                Scope::All,
                ModificationBehavior::Override,
                "COREPACK_HOME",
                cache_path,
            ))?;
        }
    }

    env = manager_layer
        .read_env()
        .map(|layer_env| layer_env.apply(Scope::Build, &env))?;

    cmd::corepack_prepare(&env).map_err(CorepackBuildpackError::CorepackPrepare)?;

    Ok(env)
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagerLayerMetadata {
    manager_name: String,
    manager_version: Version,
    layer_version: String,
}

const LAYER_VERSION: &str = "1";
