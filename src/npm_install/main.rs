use crate::common::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadata, NodeBuildScriptsMetadataError,
};
use crate::common::package_json::{PackageJson, PackageJsonError};
use crate::common::package_manager::PackageManager;
use crate::common::vrs::Version;
use crate::npm_install::configure_npm_cache_directory::configure_npm_cache_directory;
use crate::npm_install::configure_npm_runtime_env::configure_npm_runtime_env;
use crate::npm_install::{errors, npm};
use crate::{NodejsBuildpack, NodejsBuildpackError};
use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::{CmdError, CommandWithName, NamedOutput};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::Env;
use std::io;
use std::io::{stderr, Stderr};

pub(crate) fn detect(
    context: &BuildContext<NodejsBuildpack>,
) -> libcnb::Result<bool, NodejsBuildpackError> {
    context
        .app_dir
        .join(PackageManager::Npm.lockfile())
        .try_exists()
        .map_err(|e| NpmInstallBuildpackError::Detect(e).into())
}

pub(crate) fn build(
    context: &BuildContext<NodejsBuildpack>,
    env: Env,
    build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodejsBuildpackError> {
    let logger = Print::new(stderr()).h2("npm Install");
    let app_dir = &context.app_dir;
    let package_json = PackageJson::read(app_dir.join("package.json"))
        .map_err(NpmInstallBuildpackError::PackageJson)?;
    let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
        .map_err(NpmInstallBuildpackError::NodeBuildScriptsMetadata)?;

    let section = logger.bullet("Installing node modules");
    let section = log_npm_version(&env, section)?;
    let section = configure_npm_cache_directory(&context, &env, section)?;
    let section = run_npm_install(&env, section)?;
    let logger = section.done();

    let section = logger.bullet("Running scripts");
    let section = run_build_scripts(&package_json, &node_build_scripts_metadata, &env, section)?;
    let logger = section.done();

    let section = logger.bullet("Configuring default processes");
    let (build_result_builder, section) =
        configure_default_processes(&context, build_result_builder, &package_json, section);
    let logger = section.done();

    configure_npm_runtime_env(&context)?;

    logger.done();
    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmInstallBuildpackError) {
    errors::on_error(error);
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
    context: &BuildContext<NodejsBuildpack>,
    mut build_result_builder: BuildResultBuilder,
    package_json: &PackageJson,
    section_logger: Print<SubBullet<Stderr>>,
) -> (BuildResultBuilder, Print<SubBullet<Stderr>>) {
    if context.app_dir.join("Procfile").exists() {
        (
            build_result_builder,
            section_logger.sub_bullet("Skipping default web process (Procfile detected)"),
        )
    } else if package_json.has_start_script() {
        build_result_builder = build_result_builder.launch(
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), ["npm", "start"])
                        .default(true)
                        .build(),
                )
                .build(),
        );
        (
            build_result_builder,
            section_logger.sub_bullet(format!(
                "Adding default web process for {}",
                style::value("npm start")
            )),
        )
    } else {
        (
            build_result_builder,
            section_logger.sub_bullet("Skipping default web process (no start script defined)"),
        )
    }
}

#[derive(Debug)]
pub(crate) enum NpmInstallBuildpackError {
    BuildScript(CmdError),
    Detect(io::Error),
    NpmInstall(CmdError),
    NpmSetCacheDir(CmdError),
    NpmVersion(npm::VersionError),
    PackageJson(PackageJsonError),
    NodeBuildScriptsMetadata(NodeBuildScriptsMetadataError),
}

impl From<NpmInstallBuildpackError> for libcnb::Error<NodejsBuildpackError> {
    fn from(value: NpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodejsBuildpackError::NpmInstall(value))
    }
}
