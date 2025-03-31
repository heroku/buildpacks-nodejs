use crate::common::package_json::{PackageJson, PackageJsonError};
use bullet_stream::{style, Print};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::Env;
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::KeyValue;
use std::io::stderr;

use crate::corepack::cmd::CorepackVersionError;
use crate::corepack::enable_corepack::enable_corepack;
use crate::corepack::install_integrity_keys::install_integrity_keys;
use crate::corepack::prepare_corepack::prepare_corepack;
use crate::corepack::{cfg, cmd, errors};
use crate::{NodejsBuildpack, NodejsBuildpackError};

pub(crate) fn detect(
    context: &BuildContext<NodejsBuildpack>,
) -> Result<bool, libcnb::Error<NodejsBuildpackError>> {
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
    context: &BuildContext<NodejsBuildpack>,
    mut env: Env,
    build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodejsBuildpackError> {
    opentelemetry::global::tracer(context.buildpack_descriptor.buildpack.id.to_string()).in_span(
        "build",
        |cx| {
            let mut log = Print::new(stderr()).h2("Corepack");

            let pkg_mgr = PackageJson::read(context.app_dir.join("package.json"))
                .map_err(CorepackBuildpackError::PackageJson)?
                .package_manager
                .ok_or(CorepackBuildpackError::PackageManagerMissing)?;

            cx.span().set_attributes([
                KeyValue::new("package_manager.name", pkg_mgr.name.clone()),
                KeyValue::new("package_manager.version", pkg_mgr.version.to_string()),
            ]);

            let corepack_version =
                cmd::corepack_version(&env).map_err(CorepackBuildpackError::CorepackVersion)?;

            log = log
                .bullet(format!(
                    "Using Corepack version {}",
                    style::value(corepack_version.to_string()),
                ))
                .done();

            cx.span().set_attribute(KeyValue::new(
                "corepack.version",
                corepack_version.to_string(),
            ));

            log = log
                .bullet(format!(
                    "Found {package_manager_field} set to {package_manager} in {package_json}",
                    package_manager_field = style::value("packageManager"),
                    package_json = style::value("package.json"),
                    package_manager = style::value(pkg_mgr.to_string()),
                ))
                .done();

            (env, log) = enable_corepack(&context, &corepack_version, &pkg_mgr, env, log)?;
            env = install_integrity_keys(&context, &corepack_version, env)?;
            (env, log) = prepare_corepack(&context, &pkg_mgr, env, log)?;

            log.done();

            Ok((env, build_result_builder))
        },
    )
}

pub(crate) fn on_error(err: CorepackBuildpackError) {
    errors::on_error(err);
}

#[derive(Debug)]
pub(crate) enum CorepackBuildpackError {
    PackageManagerMissing,
    PackageJson(PackageJsonError),
    ShimLayer(std::io::Error),
    ManagerLayer(std::io::Error),
    CorepackVersion(CorepackVersionError),
    CorepackEnable(fun_run::CmdError),
    CorepackPrepare(fun_run::CmdError),
}

impl From<CorepackBuildpackError> for libcnb::Error<NodejsBuildpackError> {
    fn from(e: CorepackBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodejsBuildpackError::Corepack(e))
    }
}
