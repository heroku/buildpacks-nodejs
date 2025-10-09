// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::install_npm::{NpmInstallError, install_npm};
use crate::utils::error_handling::ErrorMessage;
use crate::utils::npm_registry::PackagePackument;
use crate::utils::vrs::Version;
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use libcnb::Env;
use libcnb::build::BuildResultBuilder;

pub(crate) fn build(
    context: &BuildpackBuildContext,
    mut env: Env,
    build_result_builder: BuildResultBuilder,
    npm_package_packument: &PackagePackument,
    node_version: &Version,
    existing_npm_version: &Version,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    print::bullet("Installing npm");
    if existing_npm_version == &npm_package_packument.version {
        print::sub_bullet("Requested npm version is already installed");
    } else {
        env = install_npm(context, &env, npm_package_packument, node_version)?;
    }

    let npm_version = &npm_package_packument.version;
    print::sub_bullet(format!(
        "Successfully installed {}",
        style::value(format!("npm@{npm_version}")),
    ));

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmEngineBuildpackError) -> ErrorMessage {
    super::errors::on_npm_engine_error(error)
}
#[derive(Debug)]
pub(crate) enum NpmEngineBuildpackError {
    NpmInstall(NpmInstallError),
}

impl From<NpmEngineBuildpackError> for BuildpackError {
    fn from(value: NpmEngineBuildpackError) -> Self {
        NodeJsBuildpackError::NpmEngine(value).into()
    }
}
