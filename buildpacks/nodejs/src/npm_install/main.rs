// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::npm_install::configure_npm_cache_directory::configure_npm_cache_directory;
use crate::npm_install::configure_npm_runtime_env::configure_npm_runtime_env;
use crate::npm_install::{errors, npm};
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::{CmdError, CommandWithName, NamedOutput};
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadata, NodeBuildScriptsMetadataError,
};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::package_manager::PackageManager;
use heroku_nodejs_utils::vrs::Version;
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::Env;
use std::io;

pub(crate) fn detect(
    context: &BuildContext<NodeJsBuildpack>,
) -> libcnb::Result<bool, NodeJsBuildpackError> {
    context
        .app_dir
        .join(PackageManager::Npm.lockfile())
        .try_exists()
        .map_err(|e| NpmInstallBuildpackError::Detect(e).into())
}

pub(crate) fn build(
    context: &BuildContext<NodeJsBuildpack>,
    env: Env,
    build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodeJsBuildpackError> {
    let app_dir = &context.app_dir;
    let package_json = PackageJson::read(app_dir.join("package.json"))
        .map_err(NpmInstallBuildpackError::PackageJson)?;
    let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
        .map_err(NpmInstallBuildpackError::NodeBuildScriptsMetadata)?;

    print::bullet("Installing node modules");
    log_npm_version(&env)?;
    configure_npm_cache_directory(context, &env)?;
    run_npm_install(&env)?;

    print::bullet("Running scripts");
    run_build_scripts(&package_json, &node_build_scripts_metadata, &env)?;

    print::bullet("Configuring default processes");
    let build_result_builder =
        configure_default_processes(context, build_result_builder, &package_json);

    configure_npm_runtime_env(context)?;

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmInstallBuildpackError) {
    print::plain(errors::on_npm_install_buildpack_error(error).to_string());
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
    context: &BuildContext<NodeJsBuildpack>,
    build_result_builder: BuildResultBuilder,
    package_json: &PackageJson,
) -> BuildResultBuilder {
    if context.app_dir.join("Procfile").exists() {
        print::sub_bullet("Skipping default web process (Procfile detected)");
        build_result_builder
    } else if package_json.has_start_script() {
        print::sub_bullet(format!(
            "Adding default web process for {}",
            style::value("npm start")
        ));
        build_result_builder.launch(
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), ["npm", "start"])
                        .default(true)
                        .build(),
                )
                .build(),
        )
    } else {
        print::sub_bullet("Skipping default web process (no start script defined)");
        build_result_builder
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

impl From<NpmInstallBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(value: NpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodeJsBuildpackError::NpmInstall(value))
    }
}
