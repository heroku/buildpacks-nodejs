// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::npm_engine::install_npm::{install_npm, NpmInstallError};
use crate::npm_engine::{errors, node, npm};
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use heroku_nodejs_utils::inv::{Inventory, Release};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::Env;
use std::path::Path;
use std::process::Command;

const INVENTORY: &str = include_str!("../../../../inventory/npm.toml");

pub(crate) fn detect(
    context: &BuildContext<NodeJsBuildpack>,
) -> libcnb::Result<bool, NodeJsBuildpackError> {
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
    context: &BuildContext<NodeJsBuildpack>,
    mut env: Env,
    build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodeJsBuildpackError> {
    let inventory: Inventory =
        toml::from_str(INVENTORY).map_err(NpmEngineBuildpackError::InventoryParse)?;
    let requested_npm_version = read_requested_npm_version(&context.app_dir.join("package.json"))?;
    let node_version = get_node_version(&env)?;

    print::bullet("Installing npm");
    let npm_release = resolve_requested_npm_version(&requested_npm_version, &inventory)?;
    env = install_npm(context, &env, &npm_release, &node_version)?;
    log_installed_npm_version(&env)?;

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmEngineBuildpackError) {
    print::plain(errors::on_npm_engine_buildpack_error(error).to_string());
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

fn resolve_requested_npm_version(
    requested_version: &Requirement,
    inventory: &Inventory,
) -> Result<Release, NpmEngineBuildpackError> {
    print::sub_bullet(format!(
        "Found {} version {} declared in {}",
        style::value("engines.npm"),
        style::value(requested_version.to_string()),
        style::value("package.json")
    ));

    let npm_release = inventory
        .resolve(requested_version)
        .ok_or(NpmEngineBuildpackError::NpmVersionResolve(
            requested_version.clone(),
        ))?
        .to_owned();

    print::sub_bullet(format!(
        "Resolved version {} to {}",
        style::value(requested_version.to_string()),
        style::value(npm_release.version.to_string())
    ));

    Ok(npm_release)
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

fn log_installed_npm_version(env: &Env) -> Result<(), NpmEngineBuildpackError> {
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
        .map(|npm_version| {
            print::sub_bullet(format!(
                "Successfully installed {}",
                style::value(format!("npm@{npm_version}")),
            ));
        })
}

#[derive(Debug)]
pub(crate) enum NpmEngineBuildpackError {
    PackageJson(PackageJsonError),
    MissingNpmEngineRequirement,
    InventoryParse(toml::de::Error),
    NpmVersionResolve(Requirement),
    NpmInstall(NpmInstallError),
    NodeVersion(node::VersionError),
    NpmVersion(npm::VersionError),
}

impl From<NpmEngineBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(value: NpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodeJsBuildpackError::NpmEngine(value))
    }
}
