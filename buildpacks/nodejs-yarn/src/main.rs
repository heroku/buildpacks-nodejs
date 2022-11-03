#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use crate::layers::{DepsLayer, DepsLayerError, DistLayer, DistLayerError};
use crate::yarn::Yarn;
use heroku_nodejs_utils::inv::Inventory;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::Requirement;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::layer_env::Scope;
use libcnb::{buildpack_main, Buildpack, Env};
use libherokubuildpack::log::{log_error, log_header, log_info};
use std::process::Command;
use thiserror::Error;

#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use test_support as _;
#[cfg(test)]
use ureq as _;

mod cmd;
mod layers;
mod yarn;

const INVENTORY: &str = include_str!("../inventory.toml");

pub(crate) struct NodeJsYarnBuildpack;

impl Buildpack for NodeJsYarnBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodeJsYarnBuildpackError;

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
                            .requires("yarn")
                            .provides("node_modules")
                            .requires("node_modules")
                            .requires("node")
                            .build(),
                    )
                    .build()
            })
            .unwrap_or_else(|| DetectResultBuilder::fail().build())
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header("Detecting yarn version");

        let inventory: Inventory =
            toml::from_str(INVENTORY).map_err(NodeJsYarnBuildpackError::InventoryParse)?;

        let pjson = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(NodeJsYarnBuildpackError::PackageJson)?;

        let version_range = pjson
            .engines
            .and_then(|e| {
                e.yarn.map(|v| {
                    log_info(format!(
                        "Detected yarn version range {} from package.json",
                        v
                    ));
                    v
                })
            })
            .unwrap_or_else(|| {
                log_info("Detected no yarn version range requirement");
                Requirement::any()
            });

        let release = inventory
            .resolve(&version_range)
            .ok_or(NodeJsYarnBuildpackError::YarnVersionResolve(version_range))?;

        log_info(format!("Resolved yarn version: {}", release.version));

        log_header("Installing yarn");
        let dist_layer = context.handle_layer(
            layer_name!("dist"),
            DistLayer {
                release: release.clone(),
            },
        )?;

        let env = dist_layer.env.apply(Scope::Build, &Env::from_current());

        let yarn_version =
            cmd::yarn_version(&env).map_err(NodeJsYarnBuildpackError::YarnVersionDetect)?;
        let yarn = Yarn::new(yarn_version.major())
            .map_err(NodeJsYarnBuildpackError::YarnVersionUnsupported)?;

        log_info(format!("Using yarn {yarn_version}"));

        // zero_install mode is active if the cache directory exists and is not empty
        let zero_install = cmd::yarn_get_cache(&yarn, &env)
            .map_err(NodeJsYarnBuildpackError::YarnCacheGet)?
            .read_dir()
            .map(|mut contents| {
                contents.any(|entry| {
                    entry
                        .map(|e| !e.file_name().to_string_lossy().starts_with('.'))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if zero_install {
            log_info("Yarn zero-install cache detected. Skipping dependency cache.");
        } else {
            log_header("Setting up yarn dependency cache");
            let deps_layer =
                context.handle_layer(layer_name!("deps"), DepsLayer { yarn: yarn.clone() })?;
            cmd::yarn_set_cache(&yarn, &deps_layer.path.join("cache"), &env)
                .map_err(NodeJsYarnBuildpackError::YarnCacheSet)?;
        }

        log_header("Installing dependencies");
        cmd::yarn_install(&yarn, zero_install, &env)
            .map_err(NodeJsYarnBuildpackError::YarnInstall)?;

        log_header("Running scripts");
        let build = pjson
            .scripts
            .as_ref()
            .and_then(|scripts| scripts.build.as_ref().and(Some("build")));

        match build {
            Some(key) => {
                log_info(format!("Running `{key}` script"));
                let mut proc = Command::new("yarn")
                    .args(&vec!["run", key])
                    .envs(&env)
                    .spawn()
                    .map_err(NodeJsYarnBuildpackError::BuildScript)?;
                proc.wait().map_err(NodeJsYarnBuildpackError::BuildScript)?;
            }
            None => {
                log_info("No build script found");
            }
        }

        let launch = pjson.scripts.and_then(|scripts| scripts.start).map(|_| {
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), "yarn")
                        .args(vec!["start"])
                        .default(true)
                        .build(),
                )
                .build()
        });

        match launch {
            Some(l) => BuildResultBuilder::new().launch(l).build(),
            None => BuildResultBuilder::new().build(),
        }
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        match error {
            libcnb::Error::BuildpackError(bp_err) => {
                let err_string = bp_err.to_string();
                match bp_err {
                    NodeJsYarnBuildpackError::BuildScript(_) => {
                        log_error("Yarn build script error", err_string);
                    }
                    NodeJsYarnBuildpackError::DistLayer(_) => {
                        log_error("Yarn distribution layer error", err_string);
                    }
                    NodeJsYarnBuildpackError::DepsLayer(_) => {
                        log_error("Yarn dependency layer error", err_string);
                    }
                    NodeJsYarnBuildpackError::InventoryParse(_) => {
                        log_error("Yarn inventory parse error", err_string);
                    }
                    NodeJsYarnBuildpackError::PackageJson(_) => {
                        log_error("Yarn package.json error", err_string);
                    }
                    NodeJsYarnBuildpackError::YarnCacheSet(_)
                    | NodeJsYarnBuildpackError::YarnCacheGet(_) => {
                        log_error("Yarn cache error", err_string);
                    }
                    NodeJsYarnBuildpackError::YarnInstall(_) => {
                        log_error("Yarn install error", err_string);
                    }
                    NodeJsYarnBuildpackError::YarnVersionDetect(_)
                    | NodeJsYarnBuildpackError::YarnVersionResolve(_)
                    | NodeJsYarnBuildpackError::YarnVersionUnsupported(_) => {
                        log_error("Yarn version error", err_string);
                    }
                }
            }
            err => {
                log_error("Yarn internal buildpack error", err.to_string());
            }
        }
    }
}

#[derive(Error, Debug)]
pub(crate) enum NodeJsYarnBuildpackError {
    #[error("Couldn't run build script: {0}")]
    BuildScript(std::io::Error),
    #[error("{0}")]
    DistLayer(#[from] DistLayerError),
    #[error("{0}")]
    DepsLayer(#[from] DepsLayerError),
    #[error("Couldn't parse yarn inventory: {0}")]
    InventoryParse(toml::de::Error),
    #[error("Couldn't parse package.json: {0}")]
    PackageJson(PackageJsonError),
    #[error("Couldn't read yarn cache folder: {0}")]
    YarnCacheGet(cmd::Error),
    #[error("Couldn't set yarn cache folder: {0}")]
    YarnCacheSet(cmd::Error),
    #[error("Yarn install error: {0}")]
    YarnInstall(cmd::Error),
    #[error("Couldn't determine installed yarn version: {0}")]
    YarnVersionDetect(cmd::Error),
    #[error("Unsupported yarn version: {0}")]
    YarnVersionUnsupported(std::io::Error),
    #[error("Couldn't resolve yarn version requirement ({0}) to a known yarn version")]
    YarnVersionResolve(Requirement),
}

impl From<NodeJsYarnBuildpackError> for libcnb::Error<NodeJsYarnBuildpackError> {
    fn from(e: NodeJsYarnBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsYarnBuildpack);
