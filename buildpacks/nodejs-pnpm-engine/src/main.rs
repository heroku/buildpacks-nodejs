mod errors;

use crate::errors::PnpmEngineBuildpackError;
use libcnb::build::{BuildContext, BuildResult};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;

const BUILDPACK_NAME: &str = "Heroku Node.js pnpm Engine Buildpack";

struct PnpmEngineBuildpack;

impl Buildpack for PnpmEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = PnpmEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // pass detect if a `pnpm-lock.yaml` is found
        if context.app_dir.join("pnpm-lock.yaml").exists() {
            return DetectResultBuilder::pass()
                .build_plan(
                    BuildPlanBuilder::new()
                        .provides("pnpm")
                        .requires("node")
                        .build(),
                )
                .build();
        }
        DetectResultBuilder::fail().build()
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        // This buildpack does not install pnpm yet, suggest using
        // `heroku/nodejs-corepack` instead.
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
