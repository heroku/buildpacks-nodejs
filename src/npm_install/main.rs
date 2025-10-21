// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::configure_npm_cache_directory::configure_npm_cache_directory;
use super::configure_npm_runtime_env::configure_npm_runtime_env;
use super::npm;
use crate::buildpack_config::{BuildpackConfig, ConfigValue, ConfigValueSource};
use crate::utils::error_handling::ErrorMessage;
use crate::utils::package_json::{PackageJson, PackageJsonError};
use crate::utils::vrs::Version;
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::{CmdError, CommandWithName, NamedOutput};
use indoc::indoc;
use libcnb::Env;
use libcnb::build::BuildResultBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;

pub(crate) fn build(
    context: &BuildpackBuildContext,
    env: Env,
    build_result_builder: BuildResultBuilder,
    buildpack_config: &BuildpackConfig,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    let app_dir = &context.app_dir;
    let package_json = PackageJson::read(app_dir.join("package.json"))
        .map_err(NpmInstallBuildpackError::PackageJson)?;

    print::bullet("Installing node modules");
    log_npm_version(&env)?;
    configure_npm_cache_directory(context, &env)?;
    run_npm_install(&env)?;

    print::bullet("Running scripts");
    run_build_scripts(&package_json, buildpack_config, &env)?;

    // NOTE:
    // Up until now, we haven't been pruning dev dependencies from the final runtime image. This
    // causes unnecessary bloat to image sizes so we're going to start pruning these dependencies
    // as the default behavior. Unfortunately, the front-end buildpacks rely on dev dependencies
    // being available for execution at release phase time. The front-end buildpacks also rely on
    // setting `node_build_scripts` requires metadata to configure some opt-out features. This buildpack
    // configuration mechanism is going to be changed in the near-future but, for now, we'll
    // also attach the pruning opt-out configuration to it.
    print::bullet("Pruning dev dependencies");
    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            value: false,
            source: ConfigValueSource::ProjectToml
        })
    ) {
        print::sub_bullet("Skipping as pruning was disabled in project.toml");
    } else if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            value: false,
            source: ConfigValueSource::Buildplan(_)
        })
    ) {
        print::sub_bullet("Skipping as pruning was disabled by a participating buildpack");
    } else {
        print::sub_stream_cmd(npm::Prune { env: &env }.into_command())
            .map_err(NpmInstallBuildpackError::PruneDevDependencies)?;
    }

    print::bullet("Configuring default processes");
    let build_result_builder =
        configure_default_processes(context, build_result_builder, &package_json);

    configure_npm_runtime_env(context)?;

    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            source: ConfigValueSource::ProjectToml,
            ..
        })
    ) {
        print::warning(indoc! { "
            Warning: Experimental configuration `com.heroku.buildpacks.nodejs.actions.prune_dev_dependencies` \
            found in `project.toml`. This feature may change unexpectedly in the future.
        " });
    }

    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            source: ConfigValueSource::Buildplan(_),
            ..
        })
    ) {
        print::warning(indoc! { "
            Warning: Experimental configuration `node_build_scripts.metadata.skip_pruning` was added \
            to the buildplan by a later buildpack. This feature may change unexpectedly in the future.
        " });
    }

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmInstallBuildpackError) -> ErrorMessage {
    super::errors::on_npm_install_buildpack_error(error)
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
    buildpack_config: &BuildpackConfig,
    env: &Env,
) -> Result<(), NpmInstallBuildpackError> {
    let build_scripts = package_json.build_scripts();
    if build_scripts.is_empty() {
        print::sub_bullet("No build scripts found");
    } else {
        for script in build_scripts {
            if matches!(
                buildpack_config.build_scripts_enabled,
                Some(ConfigValue {
                    value: false,
                    source: ConfigValueSource::Buildplan(_)
                })
            ) {
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
    NpmInstall(CmdError),
    NpmSetCacheDir(CmdError),
    NpmVersion(npm::VersionError),
    PackageJson(PackageJsonError),
    PruneDevDependencies(CmdError),
}

impl From<NpmInstallBuildpackError> for BuildpackError {
    fn from(value: NpmInstallBuildpackError) -> Self {
        NodeJsBuildpackError::NpmInstall(value).into()
    }
}
