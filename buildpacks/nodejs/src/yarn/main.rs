// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::yarn::cmd::YarnVersionError;
use crate::yarn::configure_yarn_cache::{configure_yarn_cache, DepsLayerError};
use crate::yarn::install_yarn::{install_yarn, CliLayerError};
use crate::yarn::{cfg, cmd, errors};
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadataError,
};
use heroku_nodejs_utils::inv::Inventory;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, VersionError};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::layer_env::Scope;
use libcnb::Env;
use serde::{Deserialize, Serialize};

const INVENTORY: &str = include_str!("../../../../inventory/yarn.toml");

const DEFAULT_YARN_REQUIREMENT: &str = "1.22.x";

#[allow(clippy::unnecessary_wraps)]
pub(crate) fn detect(
    context: &BuildContext<NodeJsBuildpack>,
) -> libcnb::Result<bool, NodeJsBuildpackError> {
    Ok(context.app_dir.join("yarn.lock").exists())
}

#[allow(clippy::too_many_lines)]
pub(crate) fn build(
    context: &BuildContext<NodeJsBuildpack>,
    mut env: Env,
    mut build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodeJsBuildpackError> {
    let pkg_json = PackageJson::read(context.app_dir.join("package.json"))
        .map_err(YarnBuildpackError::PackageJson)?;
    let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
        .map_err(YarnBuildpackError::NodeBuildScriptsMetadata)?;

    let yarn_version = match cmd::yarn_version(&env) {
        // Install yarn if it's not present.
        Err(YarnVersionError::Command(_)) => {
            print::bullet("Detecting yarn CLI version to install");

            let inventory: Inventory =
                toml::from_str(INVENTORY).map_err(YarnBuildpackError::InventoryParse)?;

            let requested_yarn_cli_range = match cfg::requested_yarn_range(&pkg_json) {
                None => {
                    print::sub_bullet(format!("No yarn engine range detected in package.json, using default ({DEFAULT_YARN_REQUIREMENT})"));
                    Requirement::parse(DEFAULT_YARN_REQUIREMENT)
                        .map_err(YarnBuildpackError::YarnDefaultParse)?
                }
                Some(requirement) => {
                    print::sub_bullet(format!(
                        "Detected yarn engine version range {requirement} in package.json"
                    ));
                    requirement
                }
            };

            let yarn_cli_release = inventory.resolve(&requested_yarn_cli_range).ok_or(
                YarnBuildpackError::YarnVersionResolve(requested_yarn_cli_range),
            )?;

            print::sub_bullet(format!(
                "Resolved yarn CLI version: {}",
                yarn_cli_release.version
            ));

            print::bullet("Installing yarn CLI");
            let yarn_env = install_yarn(context, yarn_cli_release)?;
            env = yarn_env.apply(Scope::Build, &env);

            cmd::yarn_version(&env).map_err(YarnBuildpackError::YarnVersionDetect)?
        }
        // Use the existing yarn installation if it is present.
        Ok(version) => version,
        err => err.map_err(YarnBuildpackError::YarnVersionDetect)?,
    };

    let yarn = Yarn::from_major(yarn_version.major())
        .ok_or_else(|| YarnBuildpackError::YarnVersionUnsupported(yarn_version.major()))?;

    print::bullet(format!("Yarn CLI operating in yarn {yarn_version} mode."));

    print::bullet("Setting up yarn dependency cache");
    cmd::yarn_disable_global_cache(&yarn, &env)
        .map_err(YarnBuildpackError::YarnDisableGlobalCache)?;
    let zero_install = cfg::cache_populated(
        &cmd::yarn_get_cache(&yarn, &env).map_err(YarnBuildpackError::YarnCacheGet)?,
    );
    if zero_install {
        print::sub_bullet("Yarn zero-install detected. Skipping dependency cache.");
    } else {
        configure_yarn_cache(context, &yarn, &env)?;
    }

    print::bullet("Installing dependencies");
    cmd::yarn_install(&yarn, zero_install, &env).map_err(YarnBuildpackError::YarnInstall)?;

    print::bullet("Running scripts");
    let scripts = pkg_json.build_scripts();
    if scripts.is_empty() {
        print::sub_bullet("No build scripts found");
    } else {
        for script in scripts {
            if let Some(false) = node_build_scripts_metadata.enabled {
                print::sub_bullet(format!(
                    "! Not running {script} as it was disabled by a participating buildpack",
                    script = style::value(script)
                ));
            } else {
                cmd::yarn_run(&env, &script).map_err(YarnBuildpackError::BuildScript)?;
            }
        }
    }

    if context.app_dir.join("Procfile").exists() {
        print::bullet("Skipping default web process (Procfile detected)");
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

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: YarnBuildpackError) {
    print::plain(errors::on_yarn_buildpack_error(error).to_string());
}

#[derive(Debug)]
pub(crate) enum YarnBuildpackError {
    BuildScript(fun_run::CmdError),
    CliLayer(CliLayerError),
    DepsLayer(DepsLayerError),
    InventoryParse(toml::de::Error),
    PackageJson(PackageJsonError),
    YarnCacheGet(fun_run::CmdError),
    YarnDisableGlobalCache(fun_run::CmdError),
    YarnInstall(fun_run::CmdError),
    YarnVersionDetect(YarnVersionError),
    YarnVersionUnsupported(u64),
    YarnVersionResolve(Requirement),
    YarnDefaultParse(VersionError),
    NodeBuildScriptsMetadata(NodeBuildScriptsMetadataError),
}

impl From<YarnBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(e: YarnBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodeJsBuildpackError::Yarn(e))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum Yarn {
    Yarn1,
    Yarn2,
    Yarn3,
    Yarn4,
}

impl Yarn {
    pub(crate) fn from_major(major_version: u64) -> Option<Self> {
        match major_version {
            1 => Some(Yarn::Yarn1),
            2 => Some(Yarn::Yarn2),
            3 => Some(Yarn::Yarn3),
            4 => Some(Yarn::Yarn4),
            _ => None,
        }
    }
}
