// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::pnpm_install::configure_pnpm_store_directory::configure_pnpm_store_directory;
use crate::pnpm_install::configure_pnpm_virtual_store_directory::configure_pnpm_virtual_store_directory;
use crate::pnpm_install::{cmd, errors, store};
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadataError,
};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::data::store::Store;
use libcnb::Env;
use std::path::PathBuf;
use toml::Table;

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn detect(
    context: &BuildContext<NodeJsBuildpack>,
) -> libcnb::Result<bool, NodeJsBuildpackError> {
    Ok(context.app_dir.join("pnpm-lock.yaml").exists())
}

pub(crate) fn build(
    context: &BuildContext<NodeJsBuildpack>,
    env: Env,
    mut build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodeJsBuildpackError> {
    let pkg_json = PackageJson::read(context.app_dir.join("package.json"))
        .map_err(PnpmInstallBuildpackError::PackageJson)?;
    let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
        .map_err(PnpmInstallBuildpackError::NodeBuildScriptsMetadata)?;

    print::bullet("Setting up pnpm dependency store");
    configure_pnpm_store_directory(context, &env)?;
    configure_pnpm_virtual_store_directory(context, &env)?;

    print::bullet("Installing dependencies");
    cmd::pnpm_install(&env).map_err(PnpmInstallBuildpackError::PnpmInstall)?;

    let mut metadata = if let Some(store) = &context.store {
        store.metadata.clone()
    } else {
        Table::new()
    };
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

    build_result_builder = build_result_builder.store(Store { metadata });

    if context.app_dir.join("Procfile").exists() {
        print::bullet("Skipping default web process (Procfile detected)");
    } else if pkg_json.has_start_script() {
        build_result_builder = build_result_builder.launch(
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), ["pnpm", "start"])
                        .default(true)
                        .build(),
                )
                .build(),
        );
    }

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(err: PnpmInstallBuildpackError) {
    print::plain(errors::on_pnpm_install_buildpack_error(err).to_string());
}

#[derive(Debug)]
pub(crate) enum PnpmInstallBuildpackError {
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

impl From<PnpmInstallBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodeJsBuildpackError::PnpmInstall(e))
    }
}
