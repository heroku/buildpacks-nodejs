use crate::buildpack_config::{ConfigValue, ConfigValueSource};
use crate::context::NodeJsBuildContext;
use crate::layer_cleanup::cleanup_layer;
use crate::o11y::*;
use crate::package_manager::InstalledPackageManager;
use crate::utils::error_handling::{ErrorMessage, on_framework_error};
use bullet_stream::global::print;
use indoc::indoc;
use libcnb::build::BuildResultBuilder;
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::store::Store;
use libcnb::data::{layer_name, process_type};
use libcnb::detect::DetectResultBuilder;
use libcnb::layer::UncachedLayerDefinition;
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb::{Env, additional_buildpack_binary_path, buildpack_main};
#[cfg(test)]
use libcnb_test as _;
use toml::Table;

mod buildpack_config;
mod context;
mod layer_cleanup;
mod o11y;
mod package_json;
mod package_manager;
mod package_managers;
mod runtime;
mod runtimes;
mod utils;

type BuildpackDetectContext = libcnb::detect::DetectContext<NodeJsBuildpack>;
type BuildpackBuildContext = NodeJsBuildContext;
type BuildpackError = libcnb::Error<ErrorMessage>;
type BuildpackResult<T> = Result<T, BuildpackError>;

buildpack_main!(NodeJsBuildpack);

struct NodeJsBuildpack;

impl libcnb::Buildpack for NodeJsBuildpack {
    type Platform = libcnb::generic::GenericPlatform;
    type Metadata = libcnb::generic::GenericMetadata;
    type Error = ErrorMessage;

    fn detect(
        &self,
        context: BuildpackDetectContext,
    ) -> libcnb::Result<libcnb::detect::DetectResult, ErrorMessage> {
        let buildpack_id = context.buildpack_descriptor.buildpack.id.to_string();

        // provide heroku/nodejs for other buildpacks to use
        let mut buildplan_builder = BuildPlanBuilder::new().provides(&buildpack_id);
        tracing::info!({ DETECT_PROVIDES_NODEJS } = true, "buildplan");

        // If there are common node artifacts, this buildpack should both
        // provide and require heroku/nodejs so that it may be used as
        // a standalone buildpack
        if ["package.json", "server.js", "index.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .any(|path| path.exists())
        {
            buildplan_builder = buildplan_builder.requires(&buildpack_id);
            tracing::info!({ DETECT_REQUIRES_NODEJS } = true, "buildplan");
        }

        DetectResultBuilder::pass()
            .build_plan(buildplan_builder.build())
            .build()
    }

    #[allow(clippy::too_many_lines)]
    fn build(
        &self,
        context: libcnb::build::BuildContext<NodeJsBuildpack>,
    ) -> libcnb::Result<libcnb::build::BuildResult, ErrorMessage> {
        // Wrap the context to track layers needing cleanup
        let context = NodeJsBuildContext::new(context);

        let buildpack_start = print::buildpack(
            context
                .buildpack_descriptor
                .buildpack
                .name
                .as_ref()
                .expect("The buildpack should have a name"),
        );

        let mut store = Store {
            metadata: match context.store.as_ref() {
                Some(store) => store.metadata.clone(),
                None => Table::new(),
            },
        };
        let mut build_result_builder = BuildResultBuilder::new();
        let mut env = Env::from_current();

        let buildpack_config = buildpack_config::BuildpackConfig::try_from(&context)?;

        let package_json =
            package_json::PackageJson::try_from(context.app_dir.join("package.json"))?;

        print::bullet("Checking Node.js version");
        Ok(runtime::determine_runtime(&package_json))
            .inspect(runtime::log_requested_runtime)
            .and_then(runtime::resolve_runtime)
            .inspect(runtime::log_resolved_runtime)
            .and_then(|resolved_runtime| {
                runtime::install_runtime(&context, &mut env, resolved_runtime)
            })?;

        // TODO: this code could be moved to the start of the build execution but will remain here until the package managers are cleaned up
        utils::runtime_env::register_execd_script(
            &context,
            layer_name!("web_env"),
            additional_buildpack_binary_path!("web_env"),
        )?;

        utils::runtime_env::register_execd_script(
            &context,
            layer_name!("available_parallelism"),
            additional_buildpack_binary_path!("available_parallelism"),
        )?;

        utils::build_env::set_default_env_var(
            &context,
            &mut env,
            available_parallelism::env_name(),
            available_parallelism::env_value(),
        )?;

        // TODO: this code should be moved to the end of the build execution but can't until the package managers are cleaned up
        if let Some(path) = ["server.js", "index.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .find(|path| path.exists())
        {
            build_result_builder = build_result_builder.launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(
                            process_type!("web"),
                            ["node", &path.to_string_lossy()],
                        )
                        .default(true)
                        .build(),
                    )
                    .build(),
            );
        }

        // install package manager
        let installed_package_manager = Ok(package_manager::determine_package_manager(
            &context.app_dir,
            &package_json,
        ))
        .inspect(package_manager::log_requested_package_manager)
        .and_then(|requested_package_manager| {
            package_manager::resolve_package_manager(&context, &requested_package_manager)
        })
        .inspect(package_manager::log_resolved_package_manager)
        .and_then(|resolved_package_manager| {
            package_manager::install_package_manager(&context, &mut env, &resolved_package_manager)
        })?;

        // dependency installation & process registration
        if ["pnpm-lock.yaml", "yarn.lock", "package-lock.json"]
            .iter()
            .any(|lockfile| context.app_dir.join(lockfile).exists())
        {
            package_manager::install_dependencies(
                &context,
                &env,
                &mut store,
                &installed_package_manager,
            )?;
            package_manager::run_build_scripts(
                &env,
                &installed_package_manager,
                &package_json,
                &buildpack_config,
            )?;
            package_manager::prune_dev_dependencies(
                &context,
                &env,
                &installed_package_manager,
                &buildpack_config,
            )?;

            build_result_builder = package_manager::configure_default_processes(
                &context,
                build_result_builder,
                &package_json,
                &installed_package_manager,
            );

            if matches!(installed_package_manager, InstalledPackageManager::Npm(_)) {
                // TODO: this should be done on package manager install but is current here due to how the
                //       build flow works when the bundled npm version is used
                utils::runtime_env::register_execd_script(
                    &context,
                    layer_name!("npm_runtime_config"),
                    additional_buildpack_binary_path!("npm_runtime_config"),
                )?;
            }

            if let Some(ConfigValue { source, .. }) = buildpack_config.prune_dev_dependencies {
                match source {
                    ConfigValueSource::Buildplan(_) => {
                        print::warning(indoc! { "
                            Warning: Experimental configuration `node_build_scripts.metadata.skip_pruning` was added \
                            to the buildplan by a later buildpack. This feature may change unexpectedly in the future.
                        " });
                    }
                    ConfigValueSource::ProjectToml => {
                        print::warning(indoc! { "
                            Warning: Experimental configuration `com.heroku.buildpacks.nodejs.actions.prune_dev_dependencies` \
                            found in `project.toml`. This feature may change unexpectedly in the future.
                        " });
                    }
                }
            }
        }

        let node_module_bins_layer = context.uncached_layer(
            layer_name!("z_node_module_bins"),
            UncachedLayerDefinition {
                build: true,
                launch: true,
            },
        )?;

        node_module_bins_layer.write_env(
            LayerEnv::new()
                .chainable_insert(Scope::All, ModificationBehavior::Delimiter, "PATH", ":")
                .chainable_insert(
                    Scope::All,
                    ModificationBehavior::Prepend,
                    "PATH",
                    context.app_dir.join("node_modules/.bin"),
                ),
        )?;

        // Clean up non-deterministic build artifacts from registered layers
        let layers_to_cleanup = context.layers_to_cleanup();
        if !layers_to_cleanup.is_empty() {
            for layer_to_cleanup in layers_to_cleanup {
                if let Err(e) = cleanup_layer(&layer_to_cleanup) {
                    print::sub_bullet(format!("- Error during cleanup: {e}"));
                }
            }
        }

        print::all_done(&Some(buildpack_start));

        build_result_builder.store(store).build()
    }

    fn on_error(&self, error: BuildpackError) {
        let error_message = match error {
            libcnb::Error::BuildpackError(error_message) => error_message,
            framework_error => on_framework_error(&framework_error),
        };
        tracing::error!(
            { ERROR_ID } = error_message.id,
            { ERROR_MESSAGE } = error_message.to_string(),
            "error"
        );
        print::plain(error_message.to_string());
        eprintln!();
    }
}
