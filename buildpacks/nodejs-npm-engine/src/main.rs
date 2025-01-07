mod errors;
mod install_npm;
mod node;
mod npm;

use crate::errors::NpmEngineBuildpackError;
use crate::install_npm::install_npm;
use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::CommandWithName;
use heroku_nodejs_utils::inv::{Inventory, Release};
use heroku_nodejs_utils::package_json::PackageJson;
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use serde_json as _;
use std::io::{stdout, Stdout};
use std::path::Path;
use std::process::Command;
#[cfg(test)]
use test_support as _;

const BUILDPACK_NAME: &str = "Heroku Node.js npm Engine Buildpack";

const INVENTORY: &str = include_str!("../inventory.toml");

struct NpmEngineBuildpack;

impl Buildpack for NpmEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NpmEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let package_json_path = context.app_dir.join("package.json");
        if package_json_path.exists() {
            let package_json = PackageJson::read(package_json_path)
                .map_err(NpmEngineBuildpackError::PackageJson)?;
            if package_json
                .engines
                .and_then(|engines| engines.npm)
                .is_some()
            {
                return DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .requires("npm")
                            .requires("node")
                            .provides("npm")
                            .build(),
                    )
                    .build();
            }
        }
        DetectResultBuilder::fail().build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let mut logger = Print::new(stdout()).h1(BUILDPACK_NAME);
        let env = Env::from_current();
        let inventory: Inventory =
            toml::from_str(INVENTORY).map_err(NpmEngineBuildpackError::InventoryParse)?;
        let requested_npm_version =
            read_requested_npm_version(&context.app_dir.join("package.json"))?;
        let node_version = get_node_version(&env)?;

        let section = logger.bullet("Installing npm");
        let (npm_release, section) =
            resolve_requested_npm_version(&requested_npm_version, &inventory, section)?;
        let section = install_npm(&context, &npm_release, &node_version, section)?;
        let section = log_installed_npm_version(&env, section)?;
        logger = section.done();

        logger.done();
        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        errors::on_error(error);
    }
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
    mut section_logger: Print<SubBullet<Stdout>>,
) -> Result<(Release, Print<SubBullet<Stdout>>), NpmEngineBuildpackError> {
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
    section_logger: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, NpmEngineBuildpackError> {
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

impl From<NpmEngineBuildpackError> for libcnb::Error<NpmEngineBuildpackError> {
    fn from(value: NpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(NpmEngineBuildpack);
