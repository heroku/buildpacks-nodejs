mod errors;

use crate::errors::PnpmEngineBuildpackError;
use heroku_nodejs_utils::package_json::PackageJson;
use libcnb::build::{BuildContext, BuildResult};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use serde_json as _;
#[cfg(test)]
use test_support as _;

const BUILDPACK_NAME: &str = "Heroku Node.js pnpm Engine Buildpack";

struct PnpmEngineBuildpack;

impl Buildpack for PnpmEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = PnpmEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if context.app_dir.join("pnpm-lock.yaml").exists() {
            return DetectResultBuilder::pass()
                .build_plan(
                    BuildPlanBuilder::new()
                        .requires("pnpm")
                        .requires("node")
                        .provides("pnpm")
                        .build(),
                )
                .build();
        }
        let package_json_path = context.app_dir.join("package.json");
        if package_json_path.exists() {
            let package_json = PackageJson::read(package_json_path)
                .map_err(PnpmEngineBuildpackError::PackageJson)?;
            if package_json
                .engines
                .and_then(|engines| engines.pnpm)
                .is_some()
            {
                return DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .requires("pnpm")
                            .requires("node")
                            .provides("pnpm")
                            .build(),
                    )
                    .build();
            }
        }
        DetectResultBuilder::fail().build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        Err(PnpmEngineBuildpackError::CorepackRequired)?
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        errors::on_error(error);
    }
}

impl From<PnpmEngineBuildpackError> for libcnb::Error<PnpmEngineBuildpackError> {
    fn from(value: PnpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(PnpmEngineBuildpack);
