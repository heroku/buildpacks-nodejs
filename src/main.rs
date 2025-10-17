use crate::npm_install::main::NpmInstallBuildpackError;
use crate::package_manager::RequestedPackageManager;
use crate::pnpm_install::main::PnpmInstallBuildpackError;
use crate::utils::error_handling::{ErrorMessage, on_framework_error};
use crate::yarn::main::YarnBuildpackError;
use bullet_stream::global::print;
use libcnb::build::BuildResultBuilder;
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::DetectResultBuilder;
use libcnb::{Env, additional_buildpack_binary_path, buildpack_main};
#[cfg(test)]
use libcnb_test as _;

mod npm_install;
mod package_json;
mod package_manager;
mod package_managers;
mod pnpm_install;
mod runtime;
mod runtimes;
mod utils;
mod yarn;

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

        let mut build_result_builder = BuildResultBuilder::new();
        let mut env = Env::from_current();

        let package_json =
            package_json::PackageJson::try_from(context.app_dir.join("package.json"))?;

        print::bullet("Checking Node.js version");
        let resolved_runtime = Ok(runtime::determine_runtime(&package_json))
            .inspect(runtime::log_requested_runtime)
            .and_then(runtime::resolve_runtime)
            .inspect(runtime::log_resolved_runtime)?;

        runtime::install_runtime(&context, &mut env, resolved_runtime)?;

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

        let requested_package_manager = package_manager::determine_package_manager(&package_json);

        // reproduces the group order detection logic from
        // https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/meta-buildpacks/nodejs/buildpack.toml
        if pnpm_install::main::detect(&context)?
            && requested_package_manager
                .as_ref()
                .is_some_and(RequestedPackageManager::is_pnpm)
        {
            print::bullet("Determining pnpm package information");
            if let Some(requested_package_manager) = requested_package_manager {
                env = install_package_manager(&context, env, requested_package_manager)?;
            }
            // install dependencies
            (_, build_result_builder) =
                pnpm_install::main::build(&context, env, build_result_builder)?;
        } else if detect_corepack_yarn_group(&context)? {
            if let Some(requested_package_manager) = requested_package_manager
                && requested_package_manager.is_yarn()
            {
                print::bullet("Determining Yarn information");
                env = install_package_manager(&context, env, requested_package_manager)?;
            }
            // install dependencies
            (_, build_result_builder) = yarn::main::build(&context, env, build_result_builder)?;
        } else if npm_install::main::detect(&context)? {
            if let Some(requested_package_manager) = requested_package_manager
                && requested_package_manager.is_npm()
            {
                print::bullet("Determining npm package information");
                env = install_package_manager(&context, env, requested_package_manager)?;
            }
            // install dependencies
            (_, build_result_builder) =
                npm_install::main::build(&context, env, build_result_builder)?;
        }

        print::all_done(&Some(buildpack_start));

        build_result_builder.build()
    }

    fn on_error(&self, error: BuildpackError) {
        let error_message = match error {
            libcnb::Error::BuildpackError(buildpack_error) => match buildpack_error {
                NodeJsBuildpackError::NpmInstall(error) => npm_install::main::on_error(error),
                NodeJsBuildpackError::PnpmInstall(error) => pnpm_install::main::on_error(error),
                NodeJsBuildpackError::Yarn(error) => yarn::main::on_error(error),
                NodeJsBuildpackError::Message(error) => error,
            },
            framework_error => on_framework_error(&framework_error),
        };
        print::plain(error_message.to_string());
        eprintln!();
    }
}

fn install_package_manager(
    context: &BuildpackBuildContext,
    mut env: Env,
    requested_package_manager: RequestedPackageManager,
) -> BuildpackResult<Env> {
    Ok(requested_package_manager)
        .inspect(package_manager::log_requested_package_manager)
        .and_then(|requested_package_manager| {
            package_manager::resolve_package_manager(context, &requested_package_manager)
        })
        .inspect(package_manager::log_resolved_package_manager)
        .and_then(|resolved_package_manager| {
            package_manager::install_package_manager(context, &mut env, &resolved_package_manager)
        })?;
    Ok(env)
}

// The `heroku/nodejs-engine` is already detected at the start of this buildpack since it's foundational.
//
// [[order.group]]
// id = "heroku/nodejs-engine"
//
// [[order.group]]
// id = "heroku/nodejs-corepack"
// optional = true
//
// [[order.group]]
// id = "heroku/nodejs-yarn"
fn detect_corepack_yarn_group(ctx: &BuildpackBuildContext) -> BuildpackResult<bool> {
    yarn::main::detect(ctx)
}

#[derive(Debug)]
enum NodeJsBuildpackError {
    NpmInstall(NpmInstallBuildpackError),
    PnpmInstall(PnpmInstallBuildpackError),
    Yarn(YarnBuildpackError),
    Message(ErrorMessage),
}

impl From<NodeJsBuildpackError> for BuildpackError {
    fn from(value: NodeJsBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}
