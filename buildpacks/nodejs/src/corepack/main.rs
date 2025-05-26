// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::corepack::cmd::CorepackVersionError;
use crate::corepack::enable_corepack::enable_corepack;
use crate::corepack::install_integrity_keys::install_integrity_keys;
use crate::corepack::prepare_corepack::prepare_corepack;
use crate::corepack::{cfg, cmd, errors};
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::Env;
use std::path::PathBuf;

pub(crate) fn detect(
    context: &BuildContext<NodeJsBuildpack>,
) -> Result<bool, libcnb::Error<NodeJsBuildpackError>> {
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
    context: &BuildContext<NodeJsBuildpack>,
    mut env: Env,
    build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodeJsBuildpackError> {
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

    env = enable_corepack(context, &corepack_version, &pkg_mgr, env)?;
    env = install_integrity_keys(context, &corepack_version, env)?;
    env = prepare_corepack(context, &pkg_mgr, env)?;

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(err: CorepackBuildpackError) {
    print::plain(errors::on_corepack_buildpack_error(err).to_string());
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

impl From<CorepackBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(e: CorepackBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodeJsBuildpackError::Corepack(e))
    }
}
