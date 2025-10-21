// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::cmd::{GetNodeLinkerError, YarnVersionError};
use super::configure_yarn_cache::{DepsLayerError, configure_yarn_cache};
use super::{cfg, cmd};
use crate::buildpack_config::{BuildpackConfig, ConfigValue, ConfigValueSource};
use crate::utils::error_handling::ErrorMessage;
use crate::utils::package_json::{PackageJson, PackageJsonError};
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::indoc;
use libcnb::Env;
use libcnb::build::BuildResultBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const YARN_PRUNE_PLUGIN_SOURCE: &str = include_str!("@yarnpkg/plugin-prune-dev-dependencies.js");

#[allow(clippy::too_many_lines)]
pub(crate) fn build(
    context: &BuildpackBuildContext,
    env: Env,
    mut build_result_builder: BuildResultBuilder,
    buildpack_config: &BuildpackConfig,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    let pkg_json = PackageJson::read(context.app_dir.join("package.json"))
        .map_err(YarnBuildpackError::PackageJson)?;

    let yarn_version = cmd::yarn_version(&env).map_err(YarnBuildpackError::YarnVersionDetect)?;

    let yarn = Yarn::from_major(yarn_version.major())
        .ok_or_else(|| YarnBuildpackError::YarnVersionUnsupported(yarn_version.major()))?;

    let zero_install = cfg::cache_populated(
        &cmd::yarn_get_cache(&yarn, &env).map_err(YarnBuildpackError::YarnCacheGet)?,
    );
    if zero_install {
        print::sub_bullet("Yarn zero-install detected. Skipping dependency cache.");
    } else {
        let yarn_node_linker = cmd::yarn_config_get_node_linker(&env, &yarn)
            .map_err(YarnBuildpackError::YarnGetNodeLinker)?;
        configure_yarn_cache(context, &yarn, yarn_node_linker.as_ref(), &env)?;
    }

    print::bullet("Installing dependencies");
    cmd::yarn_install(&yarn, zero_install, &env).map_err(YarnBuildpackError::YarnInstall)?;

    print::bullet("Running scripts");
    let scripts = pkg_json.build_scripts();
    if scripts.is_empty() {
        print::sub_bullet("No build scripts found");
    } else {
        for script in scripts {
            if matches!(
                buildpack_config.build_scripts_enabled,
                Some(ConfigValue {
                    value: false,
                    source: ConfigValueSource::Buildplan(_)
                })
            ) {
                print::sub_bullet(format!(
                    "! Not running {script} as it was disabled by a participating buildpack",
                    script = style::value(script)
                ));
            } else {
                cmd::yarn_run(&env, &script).map_err(YarnBuildpackError::BuildScript)?;
            }
        }
    }

    print::bullet("Pruning dev dependencies");
    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            value: false,
            source: ConfigValueSource::ProjectToml
        })
    ) {
        print::sub_bullet("Skipping as pruning was disabled in project.toml");
    } else if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            value: false,
            source: ConfigValueSource::Buildplan(_)
        })
    ) {
        print::sub_bullet("Skipping as pruning was disabled by a participating buildpack");
    } else {
        yarn_prune_dev_dependencies(&env, &yarn)?;
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

    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            source: ConfigValueSource::ProjectToml,
            ..
        })
    ) {
        print::warning(indoc! { "
            Warning: Experimental configuration `com.heroku.buildpacks.nodejs.actions.prune_dev_dependencies` \
            found in `project.toml`. This feature may change unexpectedly in the future.
        " });
    }

    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            source: ConfigValueSource::Buildplan(_),
            ..
        })
    ) {
        print::warning(indoc! { "
            Warning: Experimental configuration `node_build_scripts.metadata.skip_pruning` was added \
            to the buildplan by a later buildpack. This feature may change unexpectedly in the future.
        " });
    }

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: YarnBuildpackError) -> ErrorMessage {
    super::errors::on_yarn_error(error)
}

#[derive(Debug)]
pub(crate) enum YarnBuildpackError {
    BuildScript(fun_run::CmdError),
    DepsLayer(DepsLayerError),
    PackageJson(PackageJsonError),
    YarnCacheGet(fun_run::CmdError),
    YarnInstall(fun_run::CmdError),
    YarnVersionDetect(YarnVersionError),
    YarnVersionUnsupported(u64),
    PruneYarnDevDependencies(fun_run::CmdError),
    YarnGetNodeLinker(GetNodeLinkerError),
    InstallPrunePluginError(std::io::Error),
}

impl From<YarnBuildpackError> for BuildpackError {
    fn from(e: YarnBuildpackError) -> Self {
        NodeJsBuildpackError::Yarn(e).into()
    }
}

fn yarn_prune_dev_dependencies(env: &Env, yarn: &Yarn) -> Result<(), YarnBuildpackError> {
    match yarn {
        Yarn::Yarn1 => cmd::yarn_prune(env),
        Yarn::Yarn2 | Yarn::Yarn3 | Yarn::Yarn4 => {
            let plugin_source =
                tempfile::NamedTempFile::with_prefix("plugin-prune-dev-dependencies")
                    .map_err(YarnBuildpackError::InstallPrunePluginError)?;

            std::fs::write(plugin_source.path(), YARN_PRUNE_PLUGIN_SOURCE)
                .map_err(YarnBuildpackError::InstallPrunePluginError)?;

            cmd::yarn_prune_with_plugin(env, plugin_source.path())
        }
    }
    .map_err(YarnBuildpackError::PruneYarnDevDependencies)
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

#[derive(Debug)]
pub(crate) enum NodeLinker {
    Pnp,
    Pnpm,
    NodeModules,
}

impl FromStr for NodeLinker {
    type Err = UnknownNodeLinker;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pnp" => Ok(NodeLinker::Pnp),
            "node-modules" => Ok(NodeLinker::NodeModules),
            "pnpm" => Ok(NodeLinker::Pnpm),
            _ => Err(UnknownNodeLinker(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub(crate) struct UnknownNodeLinker(pub(crate) String);
