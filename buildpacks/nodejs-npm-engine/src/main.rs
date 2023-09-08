mod errors;
mod layers;
mod node;
mod npm;

use crate::errors::NpmEngineBuildpackError;
use crate::layers::npm_engine::NpmEngineLayer;
use commons::fun_run::CommandWithName;
use commons::output::build_log::{BuildLog, Logger, SectionLogger};
use commons::output::fmt;
use commons::output::section_log::log_step;
use heroku_nodejs_utils::inv::{Inventory, Release};
use heroku_nodejs_utils::package_json::PackageJson;
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::layer_name;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
use std::io::stdout;
use std::process::Command;

const INVENTORY: &str = include_str!("../inventory.toml");

pub(crate) struct NpmEngineBuildpack;

impl Buildpack for NpmEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NpmEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let package_json = context.app_dir.join("package.json");
        package_json
            .exists()
            .then(|| {
                PackageJson::read(package_json)
                    .map(|package_json| package_json.engines)
                    .unwrap_or(None)
                    .and_then(|engines| engines.npm)
            })
            .flatten()
            .map(|_| {
                DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .requires("npm")
                            .requires("node")
                            .provides("npm")
                            .build(),
                    )
                    .build()
            })
            .unwrap_or_else(|| DetectResultBuilder::fail().build())
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let mut logger = BuildLog::new(stdout()).buildpack_name("Heroku npm Engine Buildpack");
        let env = Env::from_current();
        let inventory: Inventory =
            toml::from_str(INVENTORY).map_err(NpmEngineBuildpackError::InventoryParse)?;
        let requested_npm_version = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(NpmEngineBuildpackError::PackageJson)
            .and_then(|package_json| {
                package_json
                    .engines
                    .and_then(|engines| engines.npm)
                    .ok_or(NpmEngineBuildpackError::MissingNpmEngineRequirement)
            })?;

        let section = logger.section("Installing npm");
        let npm_release =
            resolve_requested_npm_version(&requested_npm_version, &inventory, section.as_ref())?;
        install_npm_release(npm_release, &context, &env, section.as_ref())?;
        log_installed_npm_version(&env, section.as_ref())?;
        logger = section.end_section();

        logger.finish_logging();
        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        errors::on_error(error);
    }
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

fn install_npm_release(
    npm_release: Release,
    context: &BuildContext<NpmEngineBuildpack>,
    env: &Env,
    _section_logger: &dyn SectionLogger,
) -> Result<(), libcnb::Error<NpmEngineBuildpackError>> {
    let node_version = Command::from(node::Version { env })
        .named_output()
        .and_then(|output| output.nonzero_captured())
        .map_err(node::VersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|_| node::VersionError::Parse(stdout))
        })
        .map_err(NpmEngineBuildpackError::NodeVersion)?;

    context.handle_layer(
        layer_name!("npm_engine"),
        NpmEngineLayer {
            npm_release,
            node_version,
            _section_logger,
        },
    )?;

    Ok(())
}

fn log_installed_npm_version(
    env: &Env,
    _section_logger: &dyn SectionLogger,
) -> Result<(), NpmEngineBuildpackError> {
    Command::from(npm::Version { env })
        .named_output()
        .and_then(|output| output.nonzero_captured())
        .map_err(npm::VersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|_| npm::VersionError::Parse(stdout))
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
