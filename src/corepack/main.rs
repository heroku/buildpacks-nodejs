// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::cmd::CorepackVersionError;
use super::enable_corepack::enable_corepack;
use super::install_integrity_keys::install_integrity_keys;
use super::prepare_corepack::prepare_corepack;
use super::{cfg, cmd};
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::error_handling::ErrorMessage;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::Env;
use libcnb::build::BuildResultBuilder;
use std::path::PathBuf;

pub(crate) fn detect(context: &BuildpackBuildContext) -> BuildpackResult<bool> {
    // Corepack requires the `packageManager` key from `package.json`.
    // This buildpack won't be detected without it.
    let pkg_json_path = context.app_dir.join("package.json");
    if pkg_json_path.exists() {
        let pkg_json =
            PackageJson::read(pkg_json_path).map_err(CorepackBuildpackError::PackageJson)?;
        Ok(cfg::get_supported_package_manager(&pkg_json).is_some())
    } else {
        Ok(false)
    }
}

pub(crate) fn build(
    context: &BuildpackBuildContext,
    mut env: Env,
    build_result_builder: BuildResultBuilder,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    let pkg_mgr = PackageJson::read(context.app_dir.join("package.json"))
        .map_err(CorepackBuildpackError::PackageJson)?
        .package_manager
        .ok_or(CorepackBuildpackError::PackageManagerMissing)?;

    let corepack_version =
        cmd::corepack_version(&env).map_err(CorepackBuildpackError::CorepackVersion)?;

    print::bullet(format!(
        "Using Corepack version {}",
        style::value(corepack_version.to_string()),
    ));

    print::bullet(format!(
        "Found {package_manager_field} set to {package_manager} in {package_json}",
        package_manager_field = style::value("packageManager"),
        package_json = style::value("package.json"),
        package_manager = style::value(pkg_mgr.to_string()),
    ));

    env = enable_corepack(context, &corepack_version, &pkg_mgr, &env)?;
    env = install_integrity_keys(context, &corepack_version, env)?;
    env = prepare_corepack(context, &pkg_mgr, env)?;

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: CorepackBuildpackError) -> ErrorMessage {
    super::errors::on_corepack_error(error)
}

#[derive(Debug)]
pub(crate) enum CorepackBuildpackError {
    PackageManagerMissing,
    PackageJson(PackageJsonError),
    CreateBinDirectory(PathBuf, std::io::Error),
    CreateCacheDirectory(PathBuf, std::io::Error),
    CorepackVersion(CorepackVersionError),
    CorepackEnable(fun_run::CmdError),
    CorepackPrepare(fun_run::CmdError),
}

impl From<CorepackBuildpackError> for BuildpackError {
    fn from(e: CorepackBuildpackError) -> Self {
        NodeJsBuildpackError::Corepack(e).into()
    }
}
