#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use heroku_nodejs_utils::inv::Inventory;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::Requirement;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::log::{log_error, log_header, log_info};
use thiserror::Error;

const INVENTORY: &str = include_str!("../inventory.toml");

pub struct NodeJsYarnBuildpack;

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

        let inv: Inventory =
            toml::from_str(INVENTORY).map_err(NodeJsYarnBuildpackError::InventoryParseError)?;

        let pjson = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(NodeJsYarnBuildpackError::PackageJsonError)?;

        let version_range = pjson
            .engines
            .and_then(|e| {
                e.yarn.and_then(|v| {
                    log_info(format!(
                        "Detected yarn version range {} from package.json",
                        v
                    ));
                    Some(v)
                })
            })
            .unwrap_or_else(|| {
                log_info("Detected no yarn version range requirement");
                Requirement::any()
            });

        let target_release = inv
            .resolve(&version_range)
            .ok_or(NodeJsYarnBuildpackError::UnknownVersionError(version_range))?;

        log_info(format!("Resolved Yarn version: {}", target_release.version));

        log_header("Installing yarn");

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
                    NodeJsYarnBuildpackError::InventoryParseError(_) => {
                        log_error("Yarn inventory parse error", err_string);
                    }
                    NodeJsYarnBuildpackError::PackageJsonError(_) => {
                        log_error("Yarn package.json error", err_string);
                    }
                    NodeJsYarnBuildpackError::UnknownVersionError(_) => {
                        log_error("Yarn version error", err_string);
                    }
                }
            }
            err => {
                log_error("Internal Buildpack Error", err.to_string());
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum NodeJsYarnBuildpackError {
    #[error("Couldn't parse yarn inventory: {0}")]
    InventoryParseError(toml::de::Error),
    #[error("Couldn't parse package.json: {0}")]
    PackageJsonError(PackageJsonError),
    #[error("Couldn't resolve yarn version requirement ({0}) to a known yarn version")]
    UnknownVersionError(Requirement),
}

impl From<NodeJsYarnBuildpackError> for libcnb::Error<NodeJsYarnBuildpackError> {
    fn from(e: NodeJsYarnBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsYarnBuildpack);
