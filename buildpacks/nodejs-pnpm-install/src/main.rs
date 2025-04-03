use bullet_stream::{style, Print};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::data::store::Store;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
use std::io::stderr;

use crate::configure_pnpm_store_directory::configure_pnpm_store_directory;
use crate::configure_pnpm_virtual_store_directory::configure_pnpm_virtual_store_directory;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadataError,
    NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;

mod cmd;
mod configure_pnpm_store_directory;
mod configure_pnpm_virtual_store_directory;
mod errors;
mod store;

struct PnpmInstallBuildpack;

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
                            .provides("node_modules")
                            .provides(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                            .requires("node")
                            .requires("pnpm")
                            .requires("node_modules")
                            .requires(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                            .build(),
                    )
                    .build()
            })
            .unwrap_or_else(|| DetectResultBuilder::fail().build())
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let mut log = Print::new(stderr()).h1(context
            .buildpack_descriptor
            .buildpack
            .name
            .as_ref()
            .expect("The buildpack should have a name"));

        let env = Env::from_current();
        let pkg_json = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(PnpmInstallBuildpackError::PackageJson)?;
        let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
            .map_err(PnpmInstallBuildpackError::NodeBuildScriptsMetadata)?;

        let mut bullet = log.bullet("Setting up pnpm dependency store");
        bullet = configure_pnpm_store_directory(&context, &env, bullet)?;
        bullet = configure_pnpm_virtual_store_directory(&context, &env, bullet)?;
        log = bullet.done();

        let mut bullet = log.bullet("Installing dependencies");
        bullet = cmd::pnpm_install(&env, bullet).map_err(PnpmInstallBuildpackError::PnpmInstall)?;
        log = bullet.done();

        let mut metadata = context.store.unwrap_or_default().metadata;
        let cache_use_count = store::read_cache_use_count(&metadata);
        if store::should_prune_cache(cache_use_count) {
            let mut bullet =
                log.bullet("Pruning unused dependencies from pnpm content-addressable store");
            bullet = cmd::pnpm_store_prune(&env, bullet)
                .map_err(PnpmInstallBuildpackError::PnpmStorePrune)?;
            log = bullet.done();
        }
        store::set_cache_use_count(&mut metadata, cache_use_count + 1);

        let mut bullet = log.bullet("Running scripts");
        let scripts = pkg_json.build_scripts();
        if scripts.is_empty() {
            bullet = bullet.sub_bullet("No build scripts found");
        } else {
            for script in scripts {
                if let Some(false) = node_build_scripts_metadata.enabled {
                    bullet = bullet.sub_bullet(format!(
                        "! Not running {script} as it was disabled by a participating buildpack",
                        script = style::value(&script)
                    ));
                } else {
                    bullet = cmd::pnpm_run(&env, &script, bullet)
                        .map_err(PnpmInstallBuildpackError::BuildScript)?;
                }
            }
        }
        log = bullet.done();

        let mut result_builder = BuildResultBuilder::new().store(Store { metadata });

        if context.app_dir.join("Procfile").exists() {
            log = log
                .bullet("Skipping default web process (Procfile detected)")
                .done();
        } else if pkg_json.has_start_script() {
            result_builder = result_builder.launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(process_type!("web"), ["pnpm", "start"])
                            .default(true)
                            .build(),
                    )
                    .build(),
            );
        }

        log.done();
        result_builder.build()
    }

    fn on_error(&self, err: libcnb::Error<Self::Error>) {
        errors::on_error(err);
    }
}

#[derive(Debug)]
enum PnpmInstallBuildpackError {
    BuildScript(fun_run::CmdError),
    PackageJson(PackageJsonError),
    PnpmDir(fun_run::CmdError),
    PnpmInstall(fun_run::CmdError),
    PnpmStorePrune(fun_run::CmdError),
    VirtualLayer(std::io::Error),
    NodeBuildScriptsMetadata(NodeBuildScriptsMetadataError),
}

impl From<PnpmInstallBuildpackError> for libcnb::Error<PnpmInstallBuildpackError> {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(PnpmInstallBuildpack);
