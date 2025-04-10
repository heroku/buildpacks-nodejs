use crate::NpmInstallBuildpack;
use crate::NpmInstallBuildpackError;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::UncachedLayerDefinition;
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use std::env::temp_dir;

pub(crate) fn configure_npm_runtime_env(
    context: &BuildContext<NpmInstallBuildpack>,
) -> Result<(), libcnb::Error<NpmInstallBuildpackError>> {
    let npm_runtime_config_layer = context.uncached_layer(
        layer_name!("npm_runtime_config"),
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;

    npm_runtime_config_layer.write_env(
        LayerEnv::new()
            // We reconfigure the cache folder which, at build time points to the npm_cache layer,
            // to point to a temp folder at run time since the npm_cache layer is read-only. This
            // is helpful for two reasons:
            // - on startup, npm will try to access the cache folder
            // - npm's logs-dir is configured to also write to a directory named `_logs` inside the cache folder
            //
            // See:
            // - https://docs.npmjs.com/cli/v10/using-npm/config#cache
            // - https://docs.npmjs.com/cli/v10/using-npm/config#logs-dir
            .chainable_insert(
                Scope::Launch,
                ModificationBehavior::Override,
                "npm_config_cache",
                temp_dir().join("npm_cache"),
            )
            // Disable the update notifier at runtime which can (potentially) run at npm startup.
            // See:
            // - https://docs.npmjs.com/cli/v10/using-npm/config#update-notifier
            // - https://github.com/npm/cli/issues/7044,
            // - https://github.com/npm/cli/pull/7061
            .chainable_insert(
                Scope::Launch,
                ModificationBehavior::Override,
                "npm_config_update-notifier",
                "false",
            ),
    )
}
