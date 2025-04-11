// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod errors;

use bullet_stream::Print;
use libcnb::build::{BuildContext, BuildResult};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
#[cfg(test)]
use libcnb_test as _;
use std::io::stderr;
#[cfg(test)]
use test_support as _;

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

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let _logger = Print::new(stderr()).h1(context
            .buildpack_descriptor
            .buildpack
            .name
            .as_ref()
            .expect("The buildpack.toml should have a 'name' field set"));

        // This buildpack does not install pnpm yet, suggest using
        // `heroku/nodejs-corepack` instead.
        Err(PnpmEngineBuildpackError::CorepackRequired)?
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        let error_message = errors::on_error(error);
        eprintln!("{error_message}");
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum PnpmEngineBuildpackError {
    CorepackRequired,
}

impl From<PnpmEngineBuildpackError> for libcnb::Error<PnpmEngineBuildpackError> {
    fn from(value: PnpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(PnpmEngineBuildpack);
