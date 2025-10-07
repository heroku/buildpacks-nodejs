// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::install_node::{DistLayerError, install_node};
use crate::runtime::ResolvedRuntime;
use crate::utils::error_handling::ErrorMessage;
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use libcnb::Env;

pub(crate) fn build(
    context: &BuildpackBuildContext,
    mut env: Env,
    resolved_runtime: ResolvedRuntime,
) -> BuildpackResult<Env> {
    let ResolvedRuntime::Nodejs(target_artifact) = resolved_runtime;
    env = install_node(context, &env, &target_artifact)?;
    Ok(env)
}

pub(crate) fn on_error(error: NodeJsEngineBuildpackError) -> ErrorMessage {
    super::errors::on_nodejs_engine_error(error)
}

#[derive(Debug)]
pub(crate) enum NodeJsEngineBuildpackError {
    DistLayer(DistLayerError),
}

impl From<NodeJsEngineBuildpackError> for BuildpackError {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        NodeJsBuildpackError::NodeEngine(e).into()
    }
}
