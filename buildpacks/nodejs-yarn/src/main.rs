// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::cmd::YarnVersionError;
use crate::configure_yarn_cache::{configure_yarn_cache, DepsLayerError};
use crate::install_yarn::{install_yarn, CliLayerError};
use crate::yarn::Yarn;
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::buildplan::{
    read_node_build_scripts_metadata, NodeBuildScriptsMetadataError,
    NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::inv::Inventory;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, VersionError};
#[cfg(test)]
use indoc as _;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::layer_env::Scope;
use libcnb::{buildpack_main, Buildpack, Env};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;

mod cfg;
mod cmd;
mod configure_yarn_cache;
mod errors;
mod install_yarn;
mod yarn;

const INVENTORY: &str = include_str!("../inventory.toml");
const DEFAULT_YARN_REQUIREMENT: &str = "1.22.x";

struct YarnBuildpack;

impl Buildpack for YarnBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = YarnBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        context
            .app_dir
            .join("yarn.lock")
            .exists()
            .then(|| {
                DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .provides("yarn")
                            .provides("node_modules")
                            .provides(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                            .requires("node")
                            .requires("yarn")
                            .requires("node_modules")
                            .requires(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                            .build(),
                    )
                    .build()
            })
            .unwrap_or_else(|| DetectResultBuilder::fail().build())
    }

    #[allow(clippy::too_many_lines)]
    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let buildpack_start = print::buildpack(
            context
                .buildpack_descriptor
                .buildpack
                .name
                .as_ref()
                .expect("The buildpack should have a name"),
        );

        let mut env = Env::from_current();
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
                let yarn_env = install_yarn(&context, yarn_cli_release)?;
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
            configure_yarn_cache(&context, &yarn, &env)?;
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

        let mut build_result_builder = BuildResultBuilder::new();
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

        print::all_done(&Some(buildpack_start));
        build_result_builder.build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        let error_message = errors::on_error(error);
        eprintln!("\n{error_message}");
    }
}

#[derive(Debug)]
enum YarnBuildpackError {
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

impl From<YarnBuildpackError> for libcnb::Error<YarnBuildpackError> {
    fn from(e: YarnBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(YarnBuildpack);
