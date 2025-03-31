use crate::common::inv::Inventory;
use crate::common::package_json::{PackageJson, PackageJsonError};
use crate::common::vrs::{Requirement, VersionError};
use crate::yarn::{cfg, cmd};
use bullet_stream::{style, Print};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::layer_env::Scope;
use libcnb::Env;
use std::io::{stderr, stdout};
use thiserror::Error;

use crate::common::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadataError,
    NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use crate::yarn::configure_yarn_cache::{configure_yarn_cache, DepsLayerError};
use crate::yarn::install_yarn::{install_yarn, CliLayerError};
use crate::yarn::yarn::Yarn;
use crate::{NodejsBuildpack, NodejsBuildpackError};

const INVENTORY: &str = include_str!("../../inventory/yarn.toml");
const DEFAULT_YARN_REQUIREMENT: &str = "1.22.x";

pub(crate) fn detect(
    context: &BuildContext<NodejsBuildpack>,
) -> libcnb::Result<bool, NodejsBuildpackError> {
    Ok(context.app_dir.join("yarn.lock").exists())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn build(
    context: &BuildContext<NodejsBuildpack>,
    mut env: Env,
    mut build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodejsBuildpackError> {
    let mut log = Print::new(stderr()).h2("Yarn");

    let pkg_json = PackageJson::read(context.app_dir.join("package.json"))
        .map_err(YarnBuildpackError::PackageJson)?;
    let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
        .map_err(YarnBuildpackError::NodeBuildScriptsMetadata)?;

    let yarn_version = match cmd::yarn_version(&env) {
        // Install yarn if it's not present.
        Err(cmd::Error::Spawn(_)) => {
            let mut bullet = log.bullet("Detecting yarn CLI version to install");

            let inventory: Inventory =
                toml::from_str(INVENTORY).map_err(YarnBuildpackError::InventoryParse)?;

            let requested_yarn_cli_range = match cfg::requested_yarn_range(&pkg_json) {
                None => {
                    bullet = bullet.sub_bullet(format!("No yarn engine range detected in package.json, using default ({DEFAULT_YARN_REQUIREMENT})"));
                    Requirement::parse(DEFAULT_YARN_REQUIREMENT)
                        .map_err(YarnBuildpackError::YarnDefaultParse)?
                }
                Some(requirement) => {
                    bullet = bullet.sub_bullet(format!(
                        "Detected yarn engine version range {requirement} in package.json"
                    ));
                    requirement
                }
            };

            let yarn_cli_release = inventory.resolve(&requested_yarn_cli_range).ok_or(
                YarnBuildpackError::YarnVersionResolve(requested_yarn_cli_range),
            )?;

            bullet = bullet.sub_bullet(format!(
                "Resolved yarn CLI version: {}",
                yarn_cli_release.version
            ));
            log = bullet.done();

            bullet = log.bullet("Installing yarn CLI");
            let (yarn_env, bullet) = install_yarn(&context, yarn_cli_release, bullet)?;
            log = bullet.done();
            env = yarn_env.apply(Scope::Build, &env);

            cmd::yarn_version(&env).map_err(YarnBuildpackError::YarnVersionDetect)?
        }
        // Use the existing yarn installation if it is present.
        Ok(version) => version,
        err => err.map_err(YarnBuildpackError::YarnVersionDetect)?,
    };

    let yarn = Yarn::from_major(yarn_version.major())
        .ok_or_else(|| YarnBuildpackError::YarnVersionUnsupported(yarn_version.major()))?;

    log = log
        .bullet(format!("Yarn CLI operating in yarn {yarn_version} mode."))
        .done();

    let mut bullet = log.bullet("Setting up yarn dependency cache");
    bullet = cmd::yarn_disable_global_cache(&yarn, &env, bullet)
        .map_err(YarnBuildpackError::YarnDisableGlobalCache)?;
    let zero_install = cfg::cache_populated(
        &cmd::yarn_get_cache(&yarn, &env).map_err(YarnBuildpackError::YarnCacheGet)?,
    );
    if zero_install {
        bullet = bullet.sub_bullet("Yarn zero-install detected. Skipping dependency cache.");
    } else {
        bullet = configure_yarn_cache(&context, &yarn, &env, bullet)?;
    }
    log = bullet.done();

    let mut bullet = log.bullet("Installing dependencies");
    bullet = cmd::yarn_install(&yarn, zero_install, &env, bullet)
        .map_err(YarnBuildpackError::YarnInstall)?;
    log = bullet.done();

    let mut bullet = log.bullet("Running scripts");
    let scripts = pkg_json.build_scripts();
    if scripts.is_empty() {
        bullet = bullet.sub_bullet("No build scripts found");
    } else {
        for script in scripts {
            if let Some(false) = node_build_scripts_metadata.enabled {
                bullet = bullet.sub_bullet(format!(
                    "! Not running {script} as it was disabled by a participating buildpack",
                    script = style::value(script)
                ));
            } else {
                bullet = cmd::yarn_run(&env, &script, bullet)
                    .map_err(YarnBuildpackError::BuildScript)?;
            }
        }
    }
    log = bullet.done();

    if context.app_dir.join("Procfile").exists() {
        log = log
            .bullet("Skipping default web process (Procfile detected)")
            .done();
    } else if pkg_json.has_start_script() {
        build_result_builder = build_result_builder.launch(
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), ["yarn", "start"])
                        .default(true)
                        .build(),
                )
                .build(),
        );
    }

    log.done();
    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: YarnBuildpackError) {
    let log = Print::new(stdout()).without_header();
    match error {
        YarnBuildpackError::BuildScript(_) => {
            log.error(format!("Yarn build script error\n\n{error}"));
        }
        YarnBuildpackError::CliLayer(_) => {
            log.error(format!("Yarn distribution layer error\n\n{error}"));
        }
        YarnBuildpackError::DepsLayer(_) => {
            log.error(format!("Yarn dependency layer error\n\n{error}"));
        }
        YarnBuildpackError::InventoryParse(_) => {
            log.error(format!("Yarn inventory parse error\n\n{error}"));
        }
        YarnBuildpackError::PackageJson(_) => {
            log.error(format!("Yarn package.json error\n\n{error}"));
        }
        YarnBuildpackError::YarnCacheGet(_) | YarnBuildpackError::YarnDisableGlobalCache(_) => {
            log.error(format!("Yarn cache error\n\n{error}"));
        }
        YarnBuildpackError::YarnInstall(_) => {
            log.error(format!("Yarn install error\n\n{error}"));
        }
        YarnBuildpackError::YarnVersionDetect(_)
        | YarnBuildpackError::YarnVersionResolve(_)
        | YarnBuildpackError::YarnVersionUnsupported(_)
        | YarnBuildpackError::YarnDefaultParse(_) => {
            log.error(format!("Yarn version error\n\n{error}"));
        }
        YarnBuildpackError::NodeBuildScriptsMetadata(_) => {
            log.error(format!("Yarn buildplan error\n\n{error}"));
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum YarnBuildpackError {
    #[error("Couldn't run build script: {0}")]
    BuildScript(fun_run::CmdError),
    #[error("{0}")]
    CliLayer(#[from] CliLayerError),
    #[error("{0}")]
    DepsLayer(#[from] DepsLayerError),
    #[error("Couldn't parse yarn inventory: {0}")]
    InventoryParse(toml::de::Error),
    #[error("Couldn't parse package.json: {0}")]
    PackageJson(PackageJsonError),
    #[error("Couldn't read yarn cache folder: {0}")]
    YarnCacheGet(cmd::Error),
    #[error("Couldn't disable yarn global cache: {0}")]
    YarnDisableGlobalCache(fun_run::CmdError),
    #[error("Yarn install error: {0}")]
    YarnInstall(fun_run::CmdError),
    #[error("Couldn't determine yarn version: {0}")]
    YarnVersionDetect(cmd::Error),
    #[error("Unsupported yarn version: {0}")]
    YarnVersionUnsupported(u64),
    #[error("Couldn't resolve yarn version requirement ({0}) to a known yarn version")]
    YarnVersionResolve(Requirement),
    #[error("Couldn't parse yarn default version range: {0}")]
    YarnDefaultParse(VersionError),
    #[error("Couldn't parse metadata for the buildplan named {NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME}: {0:?}")]
    NodeBuildScriptsMetadata(NodeBuildScriptsMetadataError),
}

impl From<YarnBuildpackError> for libcnb::Error<NodejsBuildpackError> {
    fn from(e: YarnBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodejsBuildpackError::Yarn(e))
    }
}
