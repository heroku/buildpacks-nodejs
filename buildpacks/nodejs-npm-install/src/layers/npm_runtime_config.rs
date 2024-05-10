use std::env::temp_dir;
use std::path::Path;

use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb::Buildpack;

use crate::NpmInstallBuildpack;

pub(crate) struct NpmRuntimeConfigLayer;

impl Layer for NpmRuntimeConfigLayer {
    type Buildpack = NpmInstallBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: false,
            launch: true,
            cache: false,
        }
    }

    fn create(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        _layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        LayerResultBuilder::new(GenericMetadata::default())
            .env(
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
            .build()
    }
}
