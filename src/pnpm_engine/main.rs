use crate::pnpm_engine::errors;
use crate::{NodejsBuildpack, NodejsBuildpackError};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::Env;

pub(crate) fn detect(
    context: &BuildContext<NodejsBuildpack>,
) -> libcnb::Result<bool, NodejsBuildpackError> {
    // pass detect if a `pnpm-lock.yaml` is found
    Ok(context.app_dir.join("pnpm-lock.yaml").exists())
}

pub(crate) fn build(
    _context: &BuildContext<NodejsBuildpack>,
    _env: Env,
    _build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodejsBuildpackError> {
    // This buildpack does not install pnpm yet, suggest using
    // `heroku/nodejs-corepack` instead.
    Err(PnpmEngineBuildpackError::CorepackRequired)?
}

pub(crate) fn on_error(error: PnpmEngineBuildpackError) {
    errors::on_error(error);
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum PnpmEngineBuildpackError {
    CorepackRequired,
}

impl From<PnpmEngineBuildpackError> for libcnb::Error<NodejsBuildpackError> {
    fn from(value: PnpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodejsBuildpackError::PnpmEngine(value))
    }
}
