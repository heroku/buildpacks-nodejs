#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use layers::{ManagerLayer, ShimLayer};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::layer_name;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::layer_env::Scope;
use libcnb::{buildpack_main, Buildpack, Env};
use libherokubuildpack::log::log_header;

#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;
#[cfg(test)]
use ureq as _;

mod cfg;
mod cmd;
mod errors;
mod layers;

pub(crate) struct CorepackBuildpack;

impl Buildpack for CorepackBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = CorepackBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Corepack requires the `packageManager` key from `package.json`.
        // This buildpack won't be detected without it.
        let pkg_json_path = context.app_dir.join("package.json");
        if pkg_json_path.exists() {
            let pkg_json =
                PackageJson::read(pkg_json_path).map_err(CorepackBuildpackError::PackageJson)?;
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
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let pkg_mgr = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(CorepackBuildpackError::PackageJson)?
            .package_manager
            .ok_or(CorepackBuildpackError::PackageManager)?;

        let env = &Env::from_current();

        let corepack_version =
            cmd::corepack_version(env).map_err(CorepackBuildpackError::CorepackVersion)?;

        log_header(format!(
            "Installing {} {} via corepack {corepack_version}",
            pkg_mgr.name, pkg_mgr.version
        ));

        let shims_layer =
            context.handle_layer(layer_name!("shim"), ShimLayer { corepack_version })?;
        cmd::corepack_enable(&pkg_mgr.name, &shims_layer.path.join("bin"), env)
            .map_err(CorepackBuildpackError::CorepackEnable)?;

        let mgr_layer = context.handle_layer(
            layer_name!("mgr"),
            ManagerLayer {
                package_manager: pkg_mgr,
            },
        )?;
        let mgr_env = mgr_layer.env.apply(Scope::Build, env);
        cmd::corepack_prepare(&mgr_env).map_err(CorepackBuildpackError::CorepackPrepare)?;

        BuildResultBuilder::new().build()
    }

    fn on_error(&self, err: libcnb::Error<Self::Error>) {
        errors::on_error(err);
    }
}

#[derive(Debug)]
pub(crate) enum CorepackBuildpackError {
    PackageManager,
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

buildpack_main!(CorepackBuildpack);
