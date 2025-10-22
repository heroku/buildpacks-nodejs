use crate::buildpack_config::{ConfigValue, ConfigValueSource};
use crate::package_manager::InstalledPackageManager;
use crate::pnpm_install::main::PnpmInstallBuildpackError;
use crate::utils::error_handling::{ErrorMessage, on_framework_error};
use bullet_stream::global::print;
use indoc::indoc;
use libcnb::build::BuildResultBuilder;
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::store::Store;
use libcnb::data::{layer_name, process_type};
use libcnb::detect::DetectResultBuilder;
use libcnb::{Env, additional_buildpack_binary_path, buildpack_main};
#[cfg(test)]
use libcnb_test as _;
use toml::Table;

mod buildpack_config;
mod package_json;
mod package_manager;
mod package_managers;
mod pnpm_install;
mod runtime;
mod runtimes;
mod utils;

type BuildpackDetectContext = libcnb::detect::DetectContext<NodeJsBuildpack>;
type BuildpackBuildContext = libcnb::build::BuildContext<NodeJsBuildpack>;
type BuildpackError = libcnb::Error<NodeJsBuildpackError>;
type BuildpackResult<T> = Result<T, BuildpackError>;

buildpack_main!(NodeJsBuildpack);

struct NodeJsBuildpack;

impl libcnb::Buildpack for NodeJsBuildpack {
    type Platform = libcnb::generic::GenericPlatform;
    type Metadata = libcnb::generic::GenericMetadata;
    type Error = NodeJsBuildpackError;

    fn detect(
        &self,
        context: BuildpackDetectContext,
    ) -> libcnb::Result<libcnb::detect::DetectResult, NodeJsBuildpackError> {
        let buildpack_id = context.buildpack_descriptor.buildpack.id.to_string();

        // provide heroku/nodejs for other buildpacks to use
        let mut buildplan_builder = BuildPlanBuilder::new().provides(&buildpack_id);

        // If there are common node artifacts, this buildpack should both
        // provide and require heroku/nodejs so that it may be used as
        // a standalone buildpack
        if ["package.json", "server.js", "index.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .any(|path| path.exists())
        {
            buildplan_builder = buildplan_builder.requires(&buildpack_id);
        }

        DetectResultBuilder::pass()
            .build_plan(buildplan_builder.build())
            .build()
    }

    #[allow(clippy::too_many_lines)]
    fn build(
        &self,
        context: BuildpackBuildContext,
    ) -> libcnb::Result<libcnb::build::BuildResult, NodeJsBuildpackError> {
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
        if context.app_dir.join("pnpm-lock.yaml").exists() {
            package_manager::install_dependencies(
                &context,
                &env,
                &mut store,
                &installed_package_manager,
            )?;

            (_, build_result_builder) =
                pnpm_install::main::build(&context, env, build_result_builder, &buildpack_config)?;
        } else if ["yarn.lock", "package-lock.json"]
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

        print::all_done(&Some(buildpack_start));

        build_result_builder.store(store).build()
    }

    fn on_error(&self, error: BuildpackError) {
        let error_message = match error {
            libcnb::Error::BuildpackError(buildpack_error) => match buildpack_error {
                NodeJsBuildpackError::PnpmInstall(error) => pnpm_install::main::on_error(error),
                NodeJsBuildpackError::Message(error) => error,
            },
            framework_error => on_framework_error(&framework_error),
        };
        print::plain(error_message.to_string());
        eprintln!();
    }
}

#[derive(Debug)]
enum NodeJsBuildpackError {
    PnpmInstall(PnpmInstallBuildpackError),
    Message(ErrorMessage),
}

impl From<NodeJsBuildpackError> for BuildpackError {
    fn from(value: NodeJsBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}
