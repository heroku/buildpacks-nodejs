#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use crate::layers::{DistLayer, DistLayerError, WebEnvLayer};
use heroku_nodejs_utils::inv::Inventory;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::Requirement;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{Launch, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::{log_error, log_header, log_info};
use thiserror::Error;

mod layers;

#[cfg(test)]
use libcnb_test as _;

#[cfg(test)]
use ureq as _;

const INVENTORY: &str = include_str!("../inventory.toml");

pub struct NodeJsEngineBuildpack;

impl Buildpack for NodeJsEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodeJsEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let mut plan_builder = BuildPlanBuilder::new().provides("node");

        // If there are common node artifacts, this buildpack should both
        // provide and require node so that it may be used without other
        // buildpacks.
        if ["package.json", "index.js", "server.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .any(|path| path.exists())
        {
            plan_builder = plan_builder.requires("node");
        }

        // This buildpack may provide node when required by other buildpacks,
        // so it always explicitly passes. However, if no other group
        // buildpacks require node, group detection will fail.
        DetectResultBuilder::pass()
            .build_plan(plan_builder.build())
            .build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header("Heroku Node.js Engine Buildpack");
        log_header("Checking Node.js version");

        let inv: Inventory =
            toml::from_str(INVENTORY).map_err(NodeJsEngineBuildpackError::InventoryParseError)?;

        let version_range = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(NodeJsEngineBuildpackError::PackageJsonError)
            .map(|package_json| {
                package_json
                    .engines
                    .and_then(|e| e.node)
                    .unwrap_or_else(Requirement::any)
            })?;
        let version_range_string = version_range.to_string();

        log_info(format!(
            "Detected Node.js version range: {version_range_string}"
        ));

        let target_release =
            inv.resolve(&version_range)
                .ok_or(NodeJsEngineBuildpackError::UnknownVersionError(
                    version_range_string,
                ))?;

        log_info(format!(
            "Resolved Node.js version: {}",
            target_release.version
        ));

        log_header("Installing Node.js distribution");
        context.handle_layer(
            layer_name!("dist"),
            DistLayer {
                release: target_release.clone(),
            },
        )?;

        context.handle_layer(layer_name!("web_env"), WebEnvLayer)?;

        let launchjs = ["server.js", "index.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .find(|path| path.exists())
            .map(|path| {
                Launch::new().process(
                    ProcessBuilder::new(process_type!("web"), "node")
                        .args(vec![path.to_string_lossy()])
                        .default(true)
                        .build(),
                )
            });

        let resulter = BuildResultBuilder::new();
        match launchjs {
            Some(l) => resulter.launch(l).build(),
            None => resulter.build(),
        }
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) -> i32 {
        match error {
            libcnb::Error::BuildpackError(bp_err) => {
                let err_string = bp_err.to_string();
                match bp_err {
                    NodeJsEngineBuildpackError::DistLayerError(_) => {
                        log_error("Node.js engine distribution error", err_string);
                        40
                    }
                    NodeJsEngineBuildpackError::InventoryParseError(_) => {
                        log_error("Node.js engine inventory parse error", err_string);
                        41
                    }
                    NodeJsEngineBuildpackError::PackageJsonError(_) => {
                        log_error("Node.js engine package.json error", err_string);
                        42
                    }
                    NodeJsEngineBuildpackError::UnknownVersionError(_) => {
                        log_error("Node.js engine version error", err_string);
                        43
                    }
                }
            }
            err => {
                log_error("Internal Buildpack Error", err.to_string());
                100
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum NodeJsEngineBuildpackError {
    #[error("Couldn't parse Node.js inventory: {0}")]
    InventoryParseError(toml::de::Error),
    #[error("Couldn't parse package.json: {0}")]
    PackageJsonError(PackageJsonError),
    #[error("Couldn't resolve Node.js version: {0}")]
    UnknownVersionError(String),
    #[error("{0}")]
    DistLayerError(#[from] DistLayerError),
}

impl From<NodeJsEngineBuildpackError> for libcnb::Error<NodeJsEngineBuildpackError> {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsEngineBuildpack);
