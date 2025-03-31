use crate::common::buildplan::{read_node_build_scripts_metadata, NodeBuildScriptsMetadataError};
use crate::common::package_json::{PackageJson, PackageJsonError};
use crate::pnpm_install::configure_pnpm_store_directory::configure_pnpm_store_directory;
use crate::pnpm_install::configure_pnpm_virtual_store_directory::configure_pnpm_virtual_store_directory;
use crate::pnpm_install::{cmd, errors, store};
use crate::{NodejsBuildpack, NodejsBuildpackError};
use bullet_stream::{style, Print};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::data::store::Store;
use libcnb::Env;
use std::io::stderr;
use toml::Table;

pub(crate) fn detect(
    context: &BuildContext<NodejsBuildpack>,
) -> libcnb::Result<bool, NodejsBuildpackError> {
    Ok(context.app_dir.join("pnpm-lock.yaml").exists())
}

pub(crate) fn build(
    context: &BuildContext<NodejsBuildpack>,
    env: Env,
    mut build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodejsBuildpackError> {
    let mut log = Print::new(stderr()).h2("pnpm Install");

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

    let mut metadata = if let Some(store) = &context.store {
        store.metadata.clone()
    } else {
        Table::new()
    };
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

    build_result_builder = build_result_builder.store(Store { metadata });

    if context.app_dir.join("Procfile").exists() {
        log = log
            .bullet("Skipping default web process (Procfile detected)")
            .done();
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

    log.done();
    Ok((env, build_result_builder))
}

pub(crate) fn on_error(err: PnpmInstallBuildpackError) {
    errors::on_error(err);
}

#[derive(Debug)]
pub(crate) enum PnpmInstallBuildpackError {
    BuildScript(fun_run::CmdError),
    PackageJson(PackageJsonError),
    PnpmDir(fun_run::CmdError),
    PnpmInstall(fun_run::CmdError),
    PnpmStorePrune(fun_run::CmdError),
    VirtualLayer(std::io::Error),
    NodeBuildScriptsMetadata(NodeBuildScriptsMetadataError),
}

impl From<PnpmInstallBuildpackError> for libcnb::Error<NodejsBuildpackError> {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodejsBuildpackError::PnpmInstall(e))
    }
}
