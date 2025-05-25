// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::pnpm_engine::errors;
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::Env;

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn detect(
    context: &BuildContext<NodeJsBuildpack>,
) -> libcnb::Result<bool, NodeJsBuildpackError> {
    // pass detect if a `pnpm-lock.yaml` is found
    Ok(context.app_dir.join("pnpm-lock.yaml").exists())
}

pub(crate) fn build(
    _context: &BuildContext<NodeJsBuildpack>,
    _env: Env,
    _build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodeJsBuildpackError> {
    // This buildpack does not install pnpm yet, suggest using
    // `heroku/nodejs-corepack` instead.
    Err(PnpmEngineBuildpackError::CorepackRequired)?
}

pub(crate) fn on_error(error: PnpmEngineBuildpackError) {
    print::error(errors::on_pnpm_engine_buildpack_error(error).to_string());
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum PnpmEngineBuildpackError {
    CorepackRequired,
}

impl From<PnpmEngineBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(value: PnpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodeJsBuildpackError::PnpmEngine(value))
    }
}
