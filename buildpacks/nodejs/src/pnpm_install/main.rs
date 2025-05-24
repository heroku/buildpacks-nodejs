// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::configure_pnpm_store_directory::configure_pnpm_store_directory;
use crate::configure_pnpm_virtual_store_directory::configure_pnpm_virtual_store_directory;
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadataError,
    NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::data::store::Store;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
#[cfg(test)]
use libcnb_test as _;
use std::path::PathBuf;
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
        if context.app_dir.join("pnpm-lock.yaml").exists() {
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
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let buildpack_start = print::buildpack(
            context
                .buildpack_descriptor
                .buildpack
                .name
                .as_ref()
                .expect("The buildpack should have a name"),
        );

        let env = Env::from_current();
        let pkg_json = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(PnpmInstallBuildpackError::PackageJson)?;
        let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
            .map_err(PnpmInstallBuildpackError::NodeBuildScriptsMetadata)?;

        print::bullet("Setting up pnpm dependency store");
        configure_pnpm_store_directory(&context, &env)?;
        configure_pnpm_virtual_store_directory(&context, &env)?;

        print::bullet("Installing dependencies");
        cmd::pnpm_install(&env).map_err(PnpmInstallBuildpackError::PnpmInstall)?;

        let mut metadata = context.store.unwrap_or_default().metadata;
        let cache_use_count = store::read_cache_use_count(&metadata);
        if store::should_prune_cache(cache_use_count) {
            print::bullet("Pruning unused dependencies from pnpm content-addressable store");
            cmd::pnpm_store_prune(&env).map_err(PnpmInstallBuildpackError::PnpmStorePrune)?;
        }
        store::set_cache_use_count(&mut metadata, cache_use_count + 1);

        print::bullet("Running scripts");
        let scripts = pkg_json.build_scripts();
        if scripts.is_empty() {
            print::sub_bullet("No build scripts found");
        } else {
            for script in scripts {
                if let Some(false) = node_build_scripts_metadata.enabled {
                    print::sub_bullet(format!(
                        "! Not running {script} as it was disabled by a participating buildpack",
                        script = style::value(&script)
                    ));
                } else {
                    cmd::pnpm_run(&env, &script).map_err(PnpmInstallBuildpackError::BuildScript)?;
                }
            }
        }

        let mut result_builder = BuildResultBuilder::new().store(Store { metadata });

        if context.app_dir.join("Procfile").exists() {
            print::bullet("Skipping default web process (Procfile detected)");
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

        print::all_done(&Some(buildpack_start));
        result_builder.build()
    }

    fn on_error(&self, err: libcnb::Error<Self::Error>) {
        let error_message = errors::on_error(err);
        eprintln!("\n{error_message}");
    }
}

#[derive(Debug)]
enum PnpmInstallBuildpackError {
    BuildScript(fun_run::CmdError),
    PackageJson(PackageJsonError),
    PnpmSetStoreDir(fun_run::CmdError),
    PnpmSetVirtualStoreDir(fun_run::CmdError),
    PnpmInstall(fun_run::CmdError),
    PnpmStorePrune(fun_run::CmdError),
    CreateDirectory(PathBuf, std::io::Error),
    CreateSymlink {
        from: PathBuf,
        to: PathBuf,
        source: std::io::Error,
    },
    NodeBuildScriptsMetadata(NodeBuildScriptsMetadataError),
}

impl From<PnpmInstallBuildpackError> for libcnb::Error<PnpmInstallBuildpackError> {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(PnpmInstallBuildpack);
