#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]

use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use layers::{AddressableStoreLayer, VirtualStoreLayer};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
use libherokubuildpack::log::{log_header, log_info};
use serde::Deserialize;

#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;
#[cfg(test)]
use ureq as _;

mod cmd;
mod errors;
mod layers;

const CACHE_PRUNE_INTERVAL: i64 = 20;

pub(crate) struct PnpmInstallBuildpack;

impl Buildpack for PnpmInstallBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = PnpmInstallBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        context
            .app_dir
            .join("pnpm-lock.yaml")
            .exists()
            .then(|| {
                DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .provides("pnpm")
                            .requires("pnpm")
                            .provides("node_modules")
                            .requires("node_modules")
                            .requires("node")
                            .build(),
                    )
                    .build()
            })
            .unwrap_or_else(|| DetectResultBuilder::fail().build())
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let env = Env::from_current();
        let pkg_json = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(PnpmInstallBuildpackError::PackageJson)?;

        log_header("Setting up pnpm dependency store");
        let addressable_layer =
            context.handle_layer(layer_name!("addressable"), AddressableStoreLayer {})?;
        let virtual_layer = context.handle_layer(layer_name!("virtual"), VirtualStoreLayer {})?;

        cmd::pnpm_set_store_dir(&env, &addressable_layer.path)
            .map_err(PnpmInstallBuildpackError::PnpmDir)?;
        cmd::pnpm_set_virtual_dir(&env, &virtual_layer.path)
            .map_err(PnpmInstallBuildpackError::PnpmDir)?;

        log_header("Installing dependencies");
        cmd::pnpm_install(&env).map_err(PnpmInstallBuildpackError::PnpmInstall)?;

        // TODO: The content-addressable store isn't a launch layer, but it is
        // cached. The layer should be pruned to prevent unbound growth, but
        // there are advantages to allowing recently orphaned dependencies to
        // live there. Pruning every once in a while would be preferred to
        // pruning each build.
        let mut md = context.store.unwrap_or_default().metadata;
        let cache_use_count = md
            .get("cache_use_count")
            .map_or(0, |v| v.as_integer().unwrap_or(CACHE_PRUNE_INTERVAL));

        if cache_use_count.rem_euclid(CACHE_PRUNE_INTERVAL) == 0 {
            log_info("Pruning unused dependencies from pnpm content-addressable store");
            cmd::pnpm_store_prune(&env).map_err(PnpmInstallBuildpackError::PnpmStorePrune)?;
        }

        md.insert(
            "cache_use_count".to_owned(),
            toml::Value::from(cache_use_count + 1),
        );

        log_header("Running scripts");
        let scripts = pkg_json.build_scripts();
        if scripts.is_empty() {
            log_info("No build scripts found");
        } else {
            for script in scripts {
                log_info(format!("Running `{script}` script"));
                cmd::pnpm_run(&env, &script).map_err(PnpmInstallBuildpackError::BuildScript)?;
            }
        }

        if pkg_json.has_start_script() {
            BuildResultBuilder::new()
                .launch(
                    LaunchBuilder::new()
                        .process(
                            ProcessBuilder::new(process_type!("web"), "pnpm")
                                .arg("start")
                                .default(true)
                                .build(),
                        )
                        .build(),
                )
                .build()
        } else {
            BuildResultBuilder::new().build()
        }
    }

    fn on_error(&self, err: libcnb::Error<Self::Error>) {
        errors::on_error(err);
    }
}

#[derive(Debug)]
pub(crate) enum PnpmInstallBuildpackError {
    BuildScript(cmd::Error),
    PackageJson(PackageJsonError),
    PnpmDir(cmd::Error),
    PnpmInstall(cmd::Error),
    PnpmStorePrune(cmd::Error),
}

impl From<PnpmInstallBuildpackError> for libcnb::Error<PnpmInstallBuildpackError> {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(PnpmInstallBuildpack);
