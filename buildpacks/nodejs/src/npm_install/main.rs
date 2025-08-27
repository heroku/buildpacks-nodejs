// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::configure_npm_cache_directory::configure_npm_cache_directory;
use super::configure_npm_runtime_env::configure_npm_runtime_env;
use super::npm;
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::{CmdError, CommandWithName, NamedOutput};
use heroku_nodejs_utils::application;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadata, NodeBuildScriptsMetadataError,
    NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::config::{read_prune_dev_dependencies_from_project_toml, ConfigError};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::package_manager::PackageManager;
use heroku_nodejs_utils::vrs::Version;
use indoc::indoc;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use serde_json as _;
use std::io;

struct NpmInstallBuildpack;

impl Buildpack for NpmInstallBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NpmInstallBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let npm_lockfile_exists = context
            .app_dir
            .join(PackageManager::Npm.lockfile())
            .try_exists()
            .map_err(NpmInstallBuildpackError::Detect)?;

        if let Ok(package_json) = PackageJson::read(context.app_dir.join("package.json")) {
            if npm_lockfile_exists || package_json.has_dependencies() {
                DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .provides("node_modules")
                            .provides(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                            .requires("node")
                            .requires("npm")
                            .requires("node_modules")
                            .requires(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                            .build(),
                    )
                    .build()
            } else {
                DetectResultBuilder::fail().build()
            }
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let buildpack_start = print::buildpack(
            context
                .buildpack_descriptor
                .buildpack
                .name
                .as_ref()
                .expect("The buildpack.toml should have a 'name' field set"),
        );

        let env = Env::from_current();
        let app_dir = &context.app_dir;
        let package_json = PackageJson::read(app_dir.join("package.json"))
            .map_err(NpmInstallBuildpackError::PackageJson)?;
        let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
            .map_err(NpmInstallBuildpackError::NodeBuildScriptsMetadata)?;
        let prune_dev_dependencies =
            read_prune_dev_dependencies_from_project_toml(&context.app_dir.join("project.toml"))
                .map_err(NpmInstallBuildpackError::Config)?;

        application::check_for_singular_lockfile(app_dir)
            .map_err(NpmInstallBuildpackError::Application)?;

        print::bullet("Installing node modules");
        log_npm_version(&env)?;
        configure_npm_cache_directory(&context, &env)?;
        run_npm_install(&env)?;

        print::bullet("Running scripts");
        run_build_scripts(&package_json, &node_build_scripts_metadata, &env)?;

        // NOTE:
        // Up until now, we haven't been pruning dev dependencies from the final runtime image. This
        // causes unnecessary bloat to image sizes so we're going to start pruning these dependencies
        // as the default behavior. Unfortunately, the front-end buildpacks rely on dev dependencies
        // being available for execution at release phase time. The front-end buildpacks also rely on
        // setting `node_build_scripts` requires metadata to configure some opt-out features. This buildpack
        // configuration mechanism is going to be changed in the near-future but, for now, we'll
        // also attach the pruning opt-out configuration to it.
        print::bullet("Pruning dev dependencies");
        if prune_dev_dependencies == Some(false) {
            print::sub_bullet("Skipping as pruning was disabled in project.toml");
        } else if let Some(true) = node_build_scripts_metadata.skip_pruning {
            print::sub_bullet("Skipping as pruning was disabled by a participating buildpack");
        } else {
            print::sub_stream_cmd(npm::Prune { env: &env }.into_command())
                .map_err(NpmInstallBuildpackError::PruneDevDependencies)?;
        }

        print::bullet("Configuring default processes");
        let build_result = configure_default_processes(&context, &package_json);

        configure_npm_runtime_env(&context)?;

        if prune_dev_dependencies.is_some() {
            print::warning(indoc! { "
                Warning: Experimental configuration `com.heroku.buildpacks.nodejs.actions.prune_dev_dependencies` \
                found in `project.toml`. This feature may change unexpectedly in the future.
            " });
        }

        if node_build_scripts_metadata.skip_pruning.is_some() {
            print::warning(indoc! { "
                Warning: Experimental configuration `node_build_scripts.metadata.skip_pruning` was added \
                to the buildplan by a later buildpack. This feature may change unexpectedly in the future.
            " });
        }

        print::all_done(&Some(buildpack_start));
        build_result
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        let error_message = errors::on_error(error);
        eprintln!("\n{error_message}");
    }
}

fn log_npm_version(env: &Env) -> Result<(), NpmInstallBuildpackError> {
    npm::Version { env }
        .into_command()
        .named_output()
        .and_then(NamedOutput::nonzero_captured)
        .map_err(npm::VersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| npm::VersionError::Parse(stdout, e))
        })
        .map_err(NpmInstallBuildpackError::NpmVersion)
        .map(|version| {
            print::sub_bullet(format!(
                "Using npm version {}",
                style::value(version.to_string())
            ));
        })
}

fn run_npm_install(env: &Env) -> Result<(), NpmInstallBuildpackError> {
    print::sub_stream_cmd(npm::Install { env }.into_command())
        .map(|_| ())
        .map_err(NpmInstallBuildpackError::NpmInstall)
}

fn run_build_scripts(
    package_json: &PackageJson,
    node_build_scripts_metadata: &NodeBuildScriptsMetadata,
    env: &Env,
) -> Result<(), NpmInstallBuildpackError> {
    let build_scripts = package_json.build_scripts();
    if build_scripts.is_empty() {
        print::sub_bullet("No build scripts found");
    } else {
        for script in build_scripts {
            if let Some(false) = node_build_scripts_metadata.enabled {
                print::sub_bullet(format!(
                    "Not running {} as it was disabled by a participating buildpack",
                    style::value(script)
                ));
            } else {
                print::sub_stream_cmd(npm::RunScript { env, script }.into_command())
                    .map_err(NpmInstallBuildpackError::BuildScript)?;
            }
        }
    }
    Ok(())
}

fn configure_default_processes(
    context: &BuildpackBuildContext,
    package_json: &PackageJson,
) -> BuildpackResult<BuildResult> {
    if context.app_dir.join("Procfile").exists() {
        print::sub_bullet("Skipping default web process (Procfile detected)");
        BuildResultBuilder::new().build()
    } else if package_json.has_start_script() {
        print::sub_bullet(format!(
            "Adding default web process for {}",
            style::value("npm start")
        ));
        BuildResultBuilder::new()
            .launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(process_type!("web"), ["npm", "start"])
                            .default(true)
                            .build(),
                    )
                    .build(),
            )
            .build()
    } else {
        print::sub_bullet("Skipping default web process (no start script defined)");
        BuildResultBuilder::new().build()
    }
}

#[derive(Debug)]
pub(crate) enum NpmInstallBuildpackError {
    Application(application::Error),
    BuildScript(CmdError),
    Detect(io::Error),
    NpmInstall(CmdError),
    NpmSetCacheDir(CmdError),
    NpmVersion(npm::VersionError),
    PackageJson(PackageJsonError),
    NodeBuildScriptsMetadata(NodeBuildScriptsMetadataError),
    PruneDevDependencies(CmdError),
    Config(ConfigError),
}

impl From<NpmInstallBuildpackError> for BuildpackError {
    fn from(value: NpmInstallBuildpackError) -> Self {
        NodeJsBuildpackError::NpmInstall(value).into()
    }
}

buildpack_main!(NpmInstallBuildpack);
