use crate::common::inv::{Inventory, Release};
use crate::common::package_json::{PackageJson, PackageJsonError};
use crate::common::vrs::{Requirement, Version};
use crate::npm_engine::install_npm::{install_npm, NpmEngineLayerError};
use crate::npm_engine::{errors, node, npm};
use crate::{NodejsBuildpack, NodejsBuildpackError};
use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::CommandWithName;
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::Env;
use std::io::{stderr, Stderr};
use std::path::Path;
use std::process::Command;

const INVENTORY: &str = include_str!("../../inventory/npm.toml");

pub(crate) fn detect(
    context: &BuildContext<NodejsBuildpack>,
) -> libcnb::Result<bool, NodejsBuildpackError> {
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
    context: &BuildContext<NodejsBuildpack>,
    env: Env,
    build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodejsBuildpackError> {
    let mut logger = Print::new(stderr()).h2("npm Engine");
    let inventory: Inventory =
        toml::from_str(INVENTORY).map_err(NpmEngineBuildpackError::InventoryParse)?;
    let requested_npm_version = read_requested_npm_version(&context.app_dir.join("package.json"))?;
    let node_version = get_node_version(&env)?;

    let section = logger.bullet("Installing npm");
    let (npm_release, section) =
        resolve_requested_npm_version(&requested_npm_version, &inventory, section)?;
    let section = install_npm(&context, &env, &npm_release, &node_version, section)?;
    let section = log_installed_npm_version(&env, section)?;
    logger = section.done();

    logger.done();
    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmEngineBuildpackError) {
    errors::on_error(error);
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
    mut section_logger: Print<SubBullet<Stderr>>,
) -> Result<(Release, Print<SubBullet<Stderr>>), NpmEngineBuildpackError> {
    section_logger = section_logger.sub_bullet(format!(
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

    section_logger = section_logger.sub_bullet(format!(
        "Resolved version {} to {}",
        style::value(requested_version.to_string()),
        style::value(npm_release.version.to_string())
    ));

    Ok((npm_release, section_logger))
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

fn log_installed_npm_version(
    env: &Env,
    section_logger: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, NpmEngineBuildpackError> {
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
            section_logger.sub_bullet(format!(
                "Successfully installed {}",
                style::value(format!("npm@{npm_version}")),
            ))
        })
}

#[derive(Debug)]
pub(crate) enum NpmEngineBuildpackError {
    PackageJson(PackageJsonError),
    MissingNpmEngineRequirement,
    InventoryParse(toml::de::Error),
    NpmVersionResolve(Requirement),
    NpmEngineLayer(NpmEngineLayerError),
    NodeVersion(node::VersionError),
    NpmVersion(npm::VersionError),
}

impl From<NpmEngineBuildpackError> for libcnb::Error<NodejsBuildpackError> {
    fn from(value: NpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodejsBuildpackError::NpmEngine(value))
    }
}
