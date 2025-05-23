// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::cmd::CorepackVersionError;
use crate::enable_corepack::enable_corepack;
use crate::install_integrity_keys::install_integrity_keys;
use crate::prepare_corepack::prepare_corepack;
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack, Env};
#[cfg(test)]
use libcnb_test as _;
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::KeyValue;
use std::path::PathBuf;
#[cfg(test)]
use test_support as _;

mod cfg;
mod cmd;
mod enable_corepack;
mod errors;
mod install_integrity_keys;
mod prepare_corepack;

buildpack_main!(CorepackBuildpack);

struct CorepackBuildpack;

impl Buildpack for CorepackBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = CorepackBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        opentelemetry::global::tracer(context.buildpack_descriptor.buildpack.id.to_string())
            .in_span("detect", |_cx| {
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
        opentelemetry::global::tracer(context.buildpack_descriptor.buildpack.id.to_string())
            .in_span("build", |cx| {
                let buildpack_start = print::buildpack(
                    context
                        .buildpack_descriptor
                        .buildpack
                        .name
                        .as_ref()
                        .expect("The buildpack should have a name"),
                );

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

                print::bullet(format!(
                    "Using Corepack version {}",
                    style::value(corepack_version.to_string()),
                ));

                cx.span().set_attribute(KeyValue::new(
                    "corepack.version",
                    corepack_version.to_string(),
                ));

                print::bullet(format!(
                    "Found {package_manager_field} set to {package_manager} in {package_json}",
                    package_manager_field = style::value("packageManager"),
                    package_json = style::value("package.json"),
                    package_manager = style::value(pkg_mgr.to_string()),
                ));

                enable_corepack(&context, &corepack_version, &pkg_mgr, env)?;
                install_integrity_keys(&context, &corepack_version)?;
                prepare_corepack(&context, &pkg_mgr, env)?;

                print::all_done(&Some(buildpack_start));

                BuildResultBuilder::new().build()
            })
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        let error_message = errors::on_error(error);
        eprintln!("\n{error_message}");
    }
}

#[derive(Debug)]
enum CorepackBuildpackError {
    PackageManagerMissing,
    PackageJson(PackageJsonError),
    CreateBinDirectory(PathBuf, std::io::Error),
    CreateCacheDirectory(PathBuf, std::io::Error),
    CorepackVersion(CorepackVersionError),
    CorepackEnable(fun_run::CmdError),
    CorepackPrepare(fun_run::CmdError),
}

impl From<CorepackBuildpackError> for libcnb::Error<CorepackBuildpackError> {
    fn from(e: CorepackBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}
