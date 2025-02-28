mod configure_npm_cache_directory;
mod configure_npm_runtime_env;
mod errors;
mod npm;

use crate::configure_npm_cache_directory::configure_npm_cache_directory;
use crate::configure_npm_runtime_env::configure_npm_runtime_env;
use crate::errors::NpmInstallBuildpackError;
use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::{CommandWithName, NamedOutput};
use heroku_nodejs_utils::application;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadata, NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::package_json::PackageJson;
use heroku_nodejs_utils::package_manager::PackageManager;
use heroku_nodejs_utils::vrs::Version;
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
use std::io::{stderr, Stderr};
#[cfg(test)]
use test_support as _;

const BUILDPACK_NAME: &str = "Heroku Node.js npm Install Buildpack";

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
        let logger = Print::new(stderr()).h1(BUILDPACK_NAME);
        let env = Env::from_current();
        let app_dir = &context.app_dir;
        let package_json = PackageJson::read(app_dir.join("package.json"))
            .map_err(NpmInstallBuildpackError::PackageJson)?;
        let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
            .map_err(NpmInstallBuildpackError::NodeBuildScriptsMetadata)?;

        application::check_for_singular_lockfile(app_dir)
            .map_err(NpmInstallBuildpackError::Application)?;

        let section = logger.bullet("Installing node modules");
        let section = log_npm_version(&env, section)?;
        let section = configure_npm_cache_directory(&context, &env, section)?;
        let section = run_npm_install(&env, section)?;
        let logger = section.done();

        let section = logger.bullet("Running scripts");
        let section =
            run_build_scripts(&package_json, &node_build_scripts_metadata, &env, section)?;
        let logger = section.done();

        let section = logger.bullet("Configuring default processes");
        let (build_result, section) = configure_default_processes(&context, &package_json, section);
        let logger = section.done();

        configure_npm_runtime_env(&context)?;

        logger.done();
        build_result
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        errors::on_error(error);
    }
}

fn log_npm_version(
    env: &Env,
    section_logger: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, NpmInstallBuildpackError> {
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
            section_logger.sub_bullet(format!(
                "Using npm version {}",
                style::value(version.to_string())
            ))
        })
}

fn run_npm_install(
    env: &Env,
    mut section_logger: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, NpmInstallBuildpackError> {
    let mut npm_install = npm::Install { env }.into_command();
    section_logger
        .stream_with(
            format!("Running {}", style::command(npm_install.name())),
            |stdout, stderr| {
                npm_install
                    .stream_output(stdout, stderr)
                    .and_then(NamedOutput::nonzero_captured)
                    .map_err(NpmInstallBuildpackError::NpmInstall)
            },
        )
        .map(|_| section_logger)
}

fn run_build_scripts(
    package_json: &PackageJson,
    node_build_scripts_metadata: &NodeBuildScriptsMetadata,
    env: &Env,
    mut section_logger: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, NpmInstallBuildpackError> {
    let build_scripts = package_json.build_scripts();
    if build_scripts.is_empty() {
        section_logger = section_logger.sub_bullet("No build scripts found");
    } else {
        for script in build_scripts {
            if let Some(false) = node_build_scripts_metadata.enabled {
                section_logger = section_logger.sub_bullet(format!(
                    "Not running {} as it was disabled by a participating buildpack",
                    style::value(script)
                ));
            } else {
                let mut npm_run = npm::RunScript { env, script }.into_command();
                section_logger.stream_with(
                    format!("Running {}", style::command(npm_run.name())),
                    |stdout, stderr| {
                        npm_run
                            .stream_output(stdout, stderr)
                            .and_then(NamedOutput::nonzero_captured)
                            .map_err(NpmInstallBuildpackError::BuildScript)
                    },
                )?;
            }
        }
    }
    Ok(section_logger)
}

fn configure_default_processes(
    context: &BuildContext<NpmInstallBuildpack>,
    package_json: &PackageJson,
    section_logger: Print<SubBullet<Stderr>>,
) -> (
    Result<BuildResult, libcnb::Error<NpmInstallBuildpackError>>,
    Print<SubBullet<Stderr>>,
) {
    if context.app_dir.join("Procfile").exists() {
        (
            BuildResultBuilder::new().build(),
            section_logger.sub_bullet("Skipping default web process (Procfile detected)"),
        )
    } else if package_json.has_start_script() {
        (
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
                .build(),
            section_logger.sub_bullet(format!(
                "Adding default web process for {}",
                style::value("npm start")
            )),
        )
    } else {
        (
            BuildResultBuilder::new().build(),
            section_logger.sub_bullet("Skipping default web process (no start script defined)"),
        )
    }
}

impl From<NpmInstallBuildpackError> for libcnb::Error<NpmInstallBuildpackError> {
    fn from(value: NpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(NpmInstallBuildpack);
