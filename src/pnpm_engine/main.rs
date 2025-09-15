// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use heroku_nodejs_utils::error_handling::ErrorMessage;
use libcnb::Env;
use libcnb::build::BuildResultBuilder;

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn detect(context: &BuildpackBuildContext) -> BuildpackResult<bool> {
    // pass detect if a `pnpm-lock.yaml` is found
    Ok(context.app_dir.join("pnpm-lock.yaml").exists())
}

pub(crate) fn build(
    _context: &BuildpackBuildContext,
    _env: Env,
    _build_result_builder: BuildResultBuilder,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    // This buildpack does not install pnpm yet, suggest using
    // `heroku/nodejs-corepack` instead.
    Err(PnpmEngineBuildpackError::CorepackRequired)?
}

pub(crate) fn on_error(error: PnpmEngineBuildpackError) -> ErrorMessage {
    super::errors::on_pnpm_engine_error(error)
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum PnpmEngineBuildpackError {
    CorepackRequired,
}

impl From<PnpmEngineBuildpackError> for BuildpackError {
    fn from(value: PnpmEngineBuildpackError) -> Self {
        NodeJsBuildpackError::PnpmEngine(value).into()
    }
}
