use crate::NpmInstallBuildpackError;
use crate::{npm, NpmInstallBuildpack};
use bullet_stream::state::SubBullet;
use bullet_stream::Print;
use fun_run::CommandWithName;
use libcnb::build::BuildContext;
use libcnb::data::layer::LayerName;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Env;
use serde::{Deserialize, Serialize};
use std::io::Stderr;

pub(crate) fn configure_npm_cache_directory(
    buildpack: &NpmInstallBuildpack,
    context: &BuildContext<NpmInstallBuildpack>,
    env: &Env,
    mut section_logger: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, libcnb::Error<NpmInstallBuildpackError>> {
    let new_metadata = NpmCacheLayerMetadata {
        layer_version: LAYER_VERSION.to_string(),
    };

    let layer_name: LayerName = format!("{}-npm_cache", buildpack.layer_prefix)
        .parse()
        .unwrap();
    let npm_cache_layer = context.cached_layer(
        layer_name,
        CachedLayerDefinition {
            build: true,
            launch: false,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &NpmCacheLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match npm_cache_layer.state {
        LayerState::Restored { .. } => {
            section_logger = section_logger.sub_bullet("Restoring npm cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                section_logger = section_logger.sub_bullet("Restoring npm cache");
            }
            section_logger = section_logger.sub_bullet("Creating npm cache");
            npm_cache_layer.write_metadata(new_metadata)?;
        }
    }

    section_logger = section_logger.sub_bullet("Configuring npm cache directory");
    npm::SetCacheConfig {
        env,
        cache_dir: &npm_cache_layer.path(),
    }
    .into_command()
    .named_output()
    .map_err(NpmInstallBuildpackError::NpmSetCacheDir)?;

    Ok(section_logger)
}

const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NpmCacheLayerMetadata {
    layer_version: String,
}
