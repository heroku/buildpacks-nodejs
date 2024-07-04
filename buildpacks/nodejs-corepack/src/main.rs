use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::telemetry::init_tracer;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack, Env};
use libherokubuildpack::log::log_header;
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::KeyValue;

use crate::enable_corepack::enable_corepack;
use crate::prepare_corepack::prepare_corepack;
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;
#[cfg(test)]
use ureq as _;

mod cfg;
mod cmd;
mod enable_corepack;
mod errors;
mod prepare_corepack;

buildpack_main!(CorepackBuildpack);

struct CorepackBuildpack;

impl Buildpack for CorepackBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = CorepackBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let tracer = init_tracer(context.buildpack_descriptor.buildpack.id.to_string());
        tracer.in_span("nodejs-corepack-detect", |_cx| {
            // Corepack requires the `packageManager` key from `package.json`.
            // This buildpack won't be detected without it.
            let pkg_json_path = context.app_dir.join("package.json");
            if pkg_json_path.exists() {
                let pkg_json = PackageJson::read(pkg_json_path)
                    .map_err(CorepackBuildpackError::PackageJson)?;
                cfg::get_supported_package_manager(&pkg_json).map_or_else(
                    || DetectResultBuilder::fail().build(),
                    |pkg_mgr| {
                        DetectResultBuilder::pass()
                            .build_plan(
                                BuildPlanBuilder::new()
                                    .requires("node")
                                    .requires(&pkg_mgr)
                                    .provides(pkg_mgr)
                                    .build(),
                            )
                            .build()
                    },
                )
            } else {
                DetectResultBuilder::fail().build()
            }
        })
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let tracer = init_tracer(context.buildpack_descriptor.buildpack.id.to_string());
        tracer.in_span("nodejs-corepack-build", |cx| {
            let pkg_mgr = PackageJson::read(context.app_dir.join("package.json"))
                .map_err(CorepackBuildpackError::PackageJson)?
                .package_manager
                .ok_or(CorepackBuildpackError::PackageManagerMissing)?;

            cx.span().set_attributes([
                KeyValue::new("package_manager.name", pkg_mgr.name.clone()),
                KeyValue::new("package_manager.version", pkg_mgr.version.to_string()),
            ]);

            let env = &Env::from_current();

            let corepack_version =
                cmd::corepack_version(env).map_err(CorepackBuildpackError::CorepackVersion)?;

            cx.span().set_attribute(KeyValue::new(
                "corepack.version",
                corepack_version.to_string(),
            ));

            log_header(format!(
                "Installing {} {} via corepack {corepack_version}",
                pkg_mgr.name, pkg_mgr.version
            ));

            enable_corepack(&context, &corepack_version, &pkg_mgr, env)?;
            prepare_corepack(&context, &pkg_mgr, env)?;

            BuildResultBuilder::new().build()
        })
    }

    fn on_error(&self, err: libcnb::Error<Self::Error>) {
        errors::on_error(err);
    }
}

#[derive(Debug)]
enum CorepackBuildpackError {
    PackageManagerMissing,
    PackageJson(PackageJsonError),
    ShimLayer(std::io::Error),
    ManagerLayer(std::io::Error),
    CorepackVersion(cmd::Error),
    CorepackEnable(cmd::Error),
    CorepackPrepare(cmd::Error),
}

impl From<CorepackBuildpackError> for libcnb::Error<CorepackBuildpackError> {
    fn from(e: CorepackBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}
