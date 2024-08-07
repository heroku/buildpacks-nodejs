mod errors;
mod install_npm;
mod node;
mod npm;

use crate::errors::NpmEngineBuildpackError;
use crate::install_npm::install_npm;
use commons::output::build_log::{BuildLog, Logger, SectionLogger};
use commons::output::fmt;
use commons::output::section_log::log_step;
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
use std::io::stdout;
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
        let mut logger = BuildLog::new(stdout()).buildpack_name(BUILDPACK_NAME);
        let env = Env::from_current();
        let inventory: Inventory =
            toml::from_str(INVENTORY).map_err(NpmEngineBuildpackError::InventoryParse)?;
        let requested_npm_version =
            read_requested_npm_version(&context.app_dir.join("package.json"))?;
        let node_version = get_node_version(&env)?;

        let section = logger.section("Installing npm");
        let npm_release =
            resolve_requested_npm_version(&requested_npm_version, &inventory, section.as_ref())?;
        install_npm(&context, &npm_release, &node_version, section.as_ref())?;
        log_installed_npm_version(&env, section.as_ref())?;
        logger = section.end_section();

        logger.finish_logging();
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
    _section_logger: &dyn SectionLogger,
) -> Result<Release, NpmEngineBuildpackError> {
    log_step(format!(
        "Found {} version {} declared in {}",
        fmt::value("engines.npm"),
        fmt::value(requested_version.to_string()),
        fmt::value("package.json")
    ));

    let npm_release = inventory
        .resolve(requested_version)
        .ok_or(NpmEngineBuildpackError::NpmVersionResolve(
            requested_version.clone(),
        ))?
        .to_owned();

    log_step(format!(
        "Resolved version {} to {}",
        fmt::value(requested_version.to_string()),
        fmt::value(npm_release.version.to_string())
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

fn log_installed_npm_version(
    env: &Env,
    _section_logger: &dyn SectionLogger,
) -> Result<(), NpmEngineBuildpackError> {
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
            log_step(format!(
                "Successfully installed {}",
                fmt::value(format!("npm@{npm_version}")),
            ));
        })
}

impl From<NpmEngineBuildpackError> for libcnb::Error<NpmEngineBuildpackError> {
    fn from(value: NpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(NpmEngineBuildpack);
