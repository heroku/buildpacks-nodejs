use crate::yarn::Yarn;
use crate::{cmd, YarnBuildpack, YarnBuildpackError};
use bullet_stream::state::SubBullet;
use bullet_stream::Print;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Env;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Stderr;
use std::path::PathBuf;

/// `DepsLayer` is a layer for caching yarn dependencies from build to build.
/// This layer is irrelevant in zero-install mode, as cached dependencies are
/// included in the source code. Yarn only prunes unused dependencies in a few
/// scenarios, so the cache is invalidated after a number of builds to prevent
/// unbound cache growth.
pub(crate) fn configure_yarn_cache(
    context: &BuildContext<YarnBuildpack>,
    yarn: &Yarn,
    env: &Env,
    mut log: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, libcnb::Error<YarnBuildpackError>> {
    let new_metadata = DepsLayerMetadata {
        yarn: yarn.clone(),
        layer_version: LAYER_VERSION.to_string(),
        cache_usage_count: 1.0,
    };

    let deps_layer = context.cached_layer(
        layer_name!("deps"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &DepsLayerMetadata, _| {
                let is_reusable = old_metadata.yarn == new_metadata.yarn
                    && old_metadata.layer_version == new_metadata.layer_version
                    && old_metadata.cache_usage_count < MAX_CACHE_USAGE_COUNT;
                if is_reusable {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match deps_layer.state {
        LayerState::Restored { .. } => {
            log = log.sub_bullet("Restoring yarn dependency cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                log = log.sub_bullet("Clearing yarn dependency cache");
            }
            deps_layer.write_metadata(DepsLayerMetadata {
                cache_usage_count: new_metadata.cache_usage_count + 1.0,
                ..new_metadata
            })?;

            let cache_dir = deps_layer.path().join(CACHE_DIR);
            fs::create_dir(&cache_dir).map_err(|e| DepsLayerError::CreateCacheDir(cache_dir, e))?;
        }
    }

    log = cmd::yarn_set_cache(yarn, &deps_layer.path().join("cache"), env, log)
        .map_err(DepsLayerError::YarnCacheSet)?;

    Ok(log)
}

const MAX_CACHE_USAGE_COUNT: f32 = 150.0;
const CACHE_DIR: &str = "cache";
const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct DepsLayerMetadata {
    // Using float here due to [an issue with lifecycle's handling of integers](https://github.com/buildpacks/lifecycle/issues/884)
    cache_usage_count: f32,
    layer_version: String,
    yarn: Yarn,
}

#[derive(Debug)]
pub(crate) enum DepsLayerError {
    CreateCacheDir(PathBuf, std::io::Error),
    YarnCacheSet(fun_run::CmdError),
}

impl From<DepsLayerError> for libcnb::Error<YarnBuildpackError> {
    fn from(value: DepsLayerError) -> Self {
        libcnb::Error::BuildpackError(YarnBuildpackError::DepsLayer(value))
    }
}
