use commons::output::build_log::SectionLogger;
use commons::output::section_log::log_step;
use fun_run::CommandWithName;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Env;
use serde::{Deserialize, Serialize};

use crate::errors::NpmInstallBuildpackError;
use crate::{npm, NpmInstallBuildpack};

pub(crate) fn configure_npm_cache_directory(
    context: &BuildContext<NpmInstallBuildpack>,
    env: &Env,
    _section_logger: &dyn SectionLogger,
) -> Result<(), libcnb::Error<NpmInstallBuildpackError>> {
    let new_metadata = NpmCacheLayerMetadata {
        layer_version: LAYER_VERSION.to_string(),
    };

    let npm_cache_layer = context.cached_layer(
        layer_name!("npm_cache"),
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
            log_step("Restoring npm cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                log_step("Restoring npm cache");
            }
            log_step("Creating npm cache");
            npm_cache_layer.write_metadata(new_metadata)?;
        }
    }

    log_step("Configuring npm cache directory");
    npm::SetCacheConfig {
        env,
        cache_dir: &npm_cache_layer.path(),
    }
    .into_command()
    .named_output()
    .map_err(NpmInstallBuildpackError::NpmSetCacheDir)?;

    Ok(())
}

const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NpmCacheLayerMetadata {
    layer_version: String,
}
