mod cmd;
mod errors;
mod layers;

use crate::errors::NpmEngineBuildpackError;
use crate::layers::npm_engine::NpmEngineLayer;
use commons::output::build_log::{BuildLog, Logger};
use commons::output::fmt;
use heroku_nodejs_utils::inv::Inventory;
use heroku_nodejs_utils::package_json::PackageJson;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::layer_name;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
use std::io::stdout;

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

        let package_json = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(NpmEngineBuildpackError::PackageJson)?;

        let section = logger.section("Installing npm");

        let npm_requirement = package_json
            .engines
            .and_then(|engines| engines.npm)
            .ok_or(NpmEngineBuildpackError::MissingNpmEngineRequirement)?;

        let section = section.step(&format!(
            "Found {} version {} declared in {}",
            fmt::value("engines.npm"),
            fmt::value(npm_requirement.to_string()),
            fmt::value("package.json")
        ));

        let inventory: Inventory =
            toml::from_str(INVENTORY).map_err(NpmEngineBuildpackError::InventoryParse)?;

        let npm_release = inventory
            .resolve(&npm_requirement)
            .ok_or(NpmEngineBuildpackError::NpmVersionResolve(
                npm_requirement.clone(),
            ))?
            .to_owned();

        let section = section.step(&format!(
            "Resolved version {} to {}",
            fmt::value(npm_requirement.to_string()),
            fmt::value(npm_release.version.to_string())
        ));

        let node_version = cmd::node_version(&env).map_err(NpmEngineBuildpackError::Command)?;

        context.handle_layer(
            layer_name!("npm_engine"),
            NpmEngineLayer {
                npm_release,
                node_version,
                _section_logger: section.as_ref(),
            },
        )?;

        let npm_version = cmd::npm_version(&env).map_err(NpmEngineBuildpackError::Command)?;
        let section = section.step(&format!(
            "Successfully installed {}",
            fmt::value(format!("npm@{npm_version}")),
        ));

        logger = section.end_section();
        logger.finish_logging();

        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        errors::on_error(error);
    }
}

impl From<NpmEngineBuildpackError> for libcnb::Error<NpmEngineBuildpackError> {
    fn from(value: NpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(NpmEngineBuildpack);
