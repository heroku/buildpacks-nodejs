mod configure_npm_cache_directory;
mod configure_npm_runtime_env;
mod errors;
mod npm;

use crate::configure_npm_cache_directory::configure_npm_cache_directory;
use crate::configure_npm_runtime_env::configure_npm_runtime_env;
use crate::errors::NpmInstallBuildpackError;
use commons::output::build_log::{BuildLog, Logger, SectionLogger};
use commons::output::fmt;
use commons::output::section_log::{log_step, log_step_stream};
use commons::output::warn_later::WarnGuard;
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
use std::io::{stdout, Stdout};
use std::path::Path;
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
        let logger = BuildLog::new(stdout()).buildpack_name(BUILDPACK_NAME);
        let warn_later = WarnGuard::new(stdout());
        let env = Env::from_current();
        let app_dir = &context.app_dir;
        let package_json = PackageJson::read(app_dir.join("package.json"))
            .map_err(NpmInstallBuildpackError::PackageJson)?;
        let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
            .map_err(NpmInstallBuildpackError::NodeBuildScriptsMetadata)?;

        run_application_checks(app_dir, &warn_later)?;

        let section = logger.section("Installing node modules");
        log_npm_version(&env, section.as_ref())?;
        configure_npm_cache_directory(&context, &env, section.as_ref())?;
        run_npm_install(&env, section.as_ref())?;
        let logger = section.end_section();

        let section = logger.section("Running scripts");
        run_build_scripts(
            &package_json,
            &node_build_scripts_metadata,
            &env,
            section.as_ref(),
        )?;
        let logger = section.end_section();

        let section = logger.section("Configuring default processes");
        let build_result = configure_default_processes(&context, &package_json, section.as_ref());
        let logger = section.end_section();

        configure_npm_runtime_env(&context)?;

        logger.finish_logging();
        warn_later.warn_now();
        build_result
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        errors::on_error(error);
    }
}

fn run_application_checks(
    app_dir: &Path,
    warn_later: &WarnGuard<Stdout>,
) -> Result<(), NpmInstallBuildpackError> {
    application::warn_prebuilt_modules(app_dir, warn_later);
    application::check_for_singular_lockfile(app_dir).map_err(NpmInstallBuildpackError::Application)
}

fn log_npm_version(
    env: &Env,
    _section_logger: &dyn SectionLogger,
) -> Result<(), NpmInstallBuildpackError> {
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
            log_step(format!(
                "Using npm version {}",
                fmt::value(version.to_string())
            ));
        })
}

fn run_npm_install(
    env: &Env,
    _section_logger: &dyn SectionLogger,
) -> Result<(), NpmInstallBuildpackError> {
    let mut npm_install = npm::Install { env }.into_command();
    log_step_stream(
        format!("Running {}", fmt::command(npm_install.name())),
        |stream| {
            npm_install
                .stream_output(stream.io(), stream.io())
                .and_then(NamedOutput::nonzero_captured)
                .map_err(NpmInstallBuildpackError::NpmInstall)
        },
    )?;
    Ok(())
}

fn run_build_scripts(
    package_json: &PackageJson,
    node_build_scripts_metadata: &NodeBuildScriptsMetadata,
    env: &Env,
    _section_logger: &dyn SectionLogger,
) -> Result<(), NpmInstallBuildpackError> {
    let build_scripts = package_json.build_scripts();
    if build_scripts.is_empty() {
        log_step("No build scripts found");
    } else {
        for script in build_scripts {
            if let Some(false) = node_build_scripts_metadata.enabled {
                log_step(format!(
                    "Not running {} as it was disabled by a participating buildpack",
                    fmt::value(script)
                ));
            } else {
                let mut npm_run = npm::RunScript { env, script }.into_command();
                log_step_stream(
                    format!("Running {}", fmt::command(npm_run.name())),
                    |stream| {
                        npm_run
                            .stream_output(stream.io(), stream.io())
                            .and_then(NamedOutput::nonzero_captured)
                            .map_err(NpmInstallBuildpackError::BuildScript)
                    },
                )?;
            }
        }
    };
    Ok(())
}

fn configure_default_processes(
    context: &BuildContext<NpmInstallBuildpack>,
    package_json: &PackageJson,
    _section_logger: &dyn SectionLogger,
) -> Result<BuildResult, libcnb::Error<NpmInstallBuildpackError>> {
    if context.app_dir.join("Procfile").exists() {
        log_step("Skipping default web process (Procfile detected)");
        BuildResultBuilder::new().build()
    } else if package_json.has_start_script() {
        log_step(format!(
            "Adding default web process for {}",
            fmt::value("npm start")
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
        log_step("Skipping default web process (no start script defined)");
        BuildResultBuilder::new().build()
    }
}

impl From<NpmInstallBuildpackError> for libcnb::Error<NpmInstallBuildpackError> {
    fn from(value: NpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(NpmInstallBuildpack);
