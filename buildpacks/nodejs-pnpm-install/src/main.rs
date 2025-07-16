// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::cmd::PnpmVersionError;
use crate::configure_pnpm_store_directory::configure_pnpm_store_directory;
use crate::configure_pnpm_virtual_store_directory::configure_pnpm_virtual_store_directory;
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadataError,
    NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::config::{read_prune_dev_dependencies_from_project_toml, ConfigError};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use indoc::{formatdoc, indoc};
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
use serde_json::Value;
use std::path::{Path, PathBuf};
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
        let pkg_json_file = context.app_dir.join("package.json");
        let pkg_json =
            PackageJson::read(&pkg_json_file).map_err(PnpmInstallBuildpackError::PackageJson)?;
        let node_build_scripts_metadata = read_node_build_scripts_metadata(&context.buildpack_plan)
            .map_err(PnpmInstallBuildpackError::NodeBuildScriptsMetadata)?;
        let prune_dev_dependencies =
            read_prune_dev_dependencies_from_project_toml(&context.app_dir.join("project.toml"))
                .map_err(PnpmInstallBuildpackError::Config)?;
        let has_pnpm_workspace_file = has_pnpm_workspace_file(&context);

        print::bullet("Setting up pnpm dependency store");
        configure_pnpm_store_directory(&context, &env)?;
        configure_pnpm_virtual_store_directory(&context, &env)?;

        print::bullet("Installing dependencies");
        cmd::pnpm_install(&env).map_err(PnpmInstallBuildpackError::PnpmInstall)?;

        let pnpm_version = cmd::pnpm_version(&env)?;

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

        print::bullet("Pruning dev dependencies");
        if prune_dev_dependencies == Some(false) {
            print::sub_bullet("Skipping as pruning was disabled in project.toml");
        } else if node_build_scripts_metadata.skip_pruning == Some(true) {
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

        if prune_dev_dependencies.is_some() {
            print::warning(indoc! { "\
                Experimental feature used - project.toml configuration (this feature may change unexpectedly in the future)
            " });
        }

        if node_build_scripts_metadata.skip_pruning.is_some() {
            print::warning(indoc! { "\
                Experimental feature used - buildpack plan configuration (this feature may change unexpectedly in the future)
            " });
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
    PruneDevDependencies(fun_run::CmdError),
    PnpmVersion(PnpmVersionError),
    Config(ConfigError),
}

impl From<PnpmInstallBuildpackError> for libcnb::Error<PnpmInstallBuildpackError> {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(PnpmInstallBuildpack);

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

fn has_pnpm_workspace_file(context: &BuildContext<PnpmInstallBuildpack>) -> bool {
    context.app_dir.join("pnpm-workspace.yaml").exists()
        || context.app_dir.join("pnpm-workspace.yml").exists()
}
