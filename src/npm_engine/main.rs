// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::install_npm::{install_npm, NpmInstallError};
use super::{node, npm};
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use heroku_nodejs_utils::error_handling::ErrorMessage;
use heroku_nodejs_utils::npmjs_org::{
    packument_layer, resolve_package_packument, PackagePackument, PackumentLayerError,
};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::build::BuildResultBuilder;
use libcnb::Env;
use std::path::Path;
use std::process::Command;

pub(crate) fn detect(context: &BuildpackBuildContext) -> BuildpackResult<bool> {
    let package_json_path = context.app_dir.join("package.json");
    if package_json_path.exists() {
        let package_json =
            PackageJson::read(package_json_path).map_err(NpmEngineBuildpackError::PackageJson)?;
        Ok(package_json
            .engines
            .and_then(|engines| engines.npm)
            .is_some())
    } else {
        Ok(false)
    }
}

pub(crate) fn build(
    context: &BuildpackBuildContext,
    mut env: Env,
    build_result_builder: BuildResultBuilder,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    let requested_npm_version = read_requested_npm_version(&context.app_dir.join("package.json"))?;
    let node_version = get_node_version(&env)?;
    let existing_npm_version = get_npm_version(&env)?;

    print::bullet("Determining npm package information");
    let npm_package_packument = resolve_requested_npm_packument(context, &requested_npm_version)?;

    print::bullet("Installing npm");
    if existing_npm_version == npm_package_packument.version {
        print::sub_bullet("Requested npm version is already installed");
    } else {
        env = install_npm(context, &env, &npm_package_packument, &node_version)?;
    }

    let npm_version = get_npm_version(&env)?;
    print::sub_bullet(format!(
        "Successfully installed {}",
        style::value(format!("npm@{npm_version}")),
    ));

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmEngineBuildpackError) -> ErrorMessage {
    super::errors::on_npm_engine_error(error)
}

fn read_requested_npm_version(
    package_json_path: &Path,
) -> Result<Requirement, NpmEngineBuildpackError> {
    PackageJson::read(package_json_path)
        .map_err(NpmEngineBuildpackError::PackageJson)
        .and_then(|package_json| {
            package_json
                .engines
                .and_then(|engines| engines.npm)
                .ok_or(NpmEngineBuildpackError::MissingNpmEngineRequirement)
        })
}

fn resolve_requested_npm_packument(
    context: &BuildpackBuildContext,
    requested_version: &Requirement,
) -> BuildpackResult<PackagePackument> {
    print::sub_bullet(format!(
        "Found {} version {} declared in {}",
        style::value("engines.npm"),
        style::value(requested_version.to_string()),
        style::value("package.json")
    ));

    let npm_packument =
        packument_layer(context, "npm", NpmEngineBuildpackError::FetchNpmPackument)?;

    let npm_package_packument = resolve_package_packument(&npm_packument, requested_version)
        .ok_or(NpmEngineBuildpackError::NpmVersionResolve(
            requested_version.clone(),
        ))?;

    print::sub_bullet(format!(
        "Resolved npm version {} to {}",
        style::value(requested_version.to_string()),
        style::value(npm_package_packument.version.to_string())
    ));

    Ok(npm_package_packument)
}

fn get_node_version(env: &Env) -> Result<Version, NpmEngineBuildpackError> {
    Command::from(node::Version { env })
        .named_output()
        .map_err(node::VersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| node::VersionError::Parse(stdout, e))
        })
        .map_err(NpmEngineBuildpackError::NodeVersion)
}

fn get_npm_version(env: &Env) -> Result<Version, NpmEngineBuildpackError> {
    Command::from(npm::Version { env })
        .named_output()
        .map_err(npm::VersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| npm::VersionError::Parse(stdout, e))
        })
        .map_err(NpmEngineBuildpackError::NpmVersion)
}

#[derive(Debug)]
pub(crate) enum NpmEngineBuildpackError {
    PackageJson(PackageJsonError),
    MissingNpmEngineRequirement,
    FetchNpmPackument(PackumentLayerError),
    NpmVersionResolve(Requirement),
    NpmInstall(NpmInstallError),
    NodeVersion(node::VersionError),
    NpmVersion(npm::VersionError),
}

impl From<NpmEngineBuildpackError> for BuildpackError {
    fn from(value: NpmEngineBuildpackError) -> Self {
        NodeJsBuildpackError::NpmEngine(value).into()
    }
}
