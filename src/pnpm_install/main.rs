// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::cmd;
use super::cmd::PnpmVersionError;
use crate::buildpack_config::{BuildpackConfig, ConfigValue, ConfigValueSource};
use crate::utils::error_handling::ErrorMessage;
use crate::utils::package_json::{PackageJson, PackageJsonError};
use crate::utils::vrs::{Requirement, Version};
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::{formatdoc, indoc};
use libcnb::Env;
use libcnb::build::BuildResultBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use serde_json::Value;
use std::path::Path;

#[allow(clippy::too_many_lines)]
pub(crate) fn build(
    context: &BuildpackBuildContext,
    env: Env,
    mut build_result_builder: BuildResultBuilder,
    buildpack_config: &BuildpackConfig,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    let pkg_json_file = context.app_dir.join("package.json");
    let pkg_json =
        PackageJson::read(&pkg_json_file).map_err(PnpmInstallBuildpackError::PackageJson)?;
    let has_pnpm_workspace_file = has_pnpm_workspace_file(context);

    let pnpm_version = cmd::pnpm_version(&env)?;

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
        pnpm_prune_dev_dependencies(
            &env,
            &pnpm_version,
            Requirement::parse(">8.15.6")
                .expect("Should be valid range")
                .satisfies(&pnpm_version),
            has_lifecycle_scripts(&pkg_json_file)
                .map_err(PnpmInstallBuildpackError::PackageJson)?,
            has_pnpm_workspace_file,
        )?;
    }

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

pub(crate) fn on_error(error: PnpmInstallBuildpackError) -> ErrorMessage {
    super::errors::on_pnpm_install_buildpack_error(error)
}

#[derive(Debug)]
pub(crate) enum PnpmInstallBuildpackError {
    PackageJson(PackageJsonError),
    PruneDevDependencies(fun_run::CmdError),
    PnpmVersion(PnpmVersionError),
}

impl From<PnpmInstallBuildpackError> for BuildpackError {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        NodeJsBuildpackError::PnpmInstall(e).into()
    }
}

fn pnpm_prune_dev_dependencies(
    env: &Env,
    pnpm_version: &Version,
    supports_ignore_script_flag: bool,
    has_lifecycle_scripts: bool,
    has_workspace_file: bool,
) -> Result<(), PnpmInstallBuildpackError> {
    if has_workspace_file {
        print::sub_bullet(format!(
            "Skipping because pruning is not supported for pnpm workspaces ({})",
            style::url("https://pnpm.io/cli/prune")
        ));
        return Ok(());
    }

    if supports_ignore_script_flag {
        return cmd::pnpm_prune_dev_dependencies(env, vec!["--ignore-scripts"])
            .map_err(PnpmInstallBuildpackError::PruneDevDependencies);
    }

    if has_lifecycle_scripts {
        print::warning(formatdoc! { "
            Pruning skipped due to presence of lifecycle scripts
        
            The version of pnpm used ({pnpm_version}) will execute the following lifecycle scripts \
            declared in package.json during pruning which can cause build failures:
            - pnpm:devPreinstall
            - preinstall
            - install
            - postinstall
            - prepare
        
            Since pruning can't be done safely for your build, it will be skipped. To fix this you \
            must upgrade your version of pnpm to 8.15.6 or higher.
        "});
        Ok(())
    } else {
        cmd::pnpm_prune_dev_dependencies(env, vec![])
            .map_err(PnpmInstallBuildpackError::PruneDevDependencies)
    }
}

fn has_lifecycle_scripts(package_json_file: &Path) -> Result<bool, PackageJsonError> {
    let lifecycle_scripts = ["pnpm:devPreinstall", "preinstall", "postinstall", "prepare"];
    let contents =
        std::fs::read_to_string(package_json_file).map_err(PackageJsonError::AccessError)?;
    let json = serde_json::from_str::<Value>(&contents).map_err(PackageJsonError::ParseError)?;
    Ok(
        if let Some(scripts) = json.get("scripts").and_then(|scripts| scripts.as_object()) {
            scripts
                .keys()
                .any(|script_name| lifecycle_scripts.contains(&script_name.as_str()))
        } else {
            false
        },
    )
}

fn has_pnpm_workspace_file(context: &BuildpackBuildContext) -> bool {
    context.app_dir.join("pnpm-workspace.yaml").exists()
        || context.app_dir.join("pnpm-workspace.yml").exists()
}
