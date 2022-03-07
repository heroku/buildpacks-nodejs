#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{Launch, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack};
use libherokubuildpack::{log_error, log_header, log_info};
use libhkcnb_nodejs::inv::Inventory;
use libhkcnb_nodejs::package_json::{PackageJson, PackageJsonError};
use libhkcnb_nodejs::vrs::Requirement;
use std::path::Path;
use thiserror::Error;

use crate::layers::{DistLayer, DistLayerError, WebEnvLayer};

mod layers;

const INVENTORY: &str = include_str!("../inventory.toml");

pub struct NodeJsEngineBuildpack;

impl Buildpack for NodeJsEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodeJsEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let pass_result = DetectResultBuilder::pass();
        let plan_builder = BuildPlanBuilder::new().provides("node");
        let app_dir = context.app_dir;

        // If there are common node artifacts, this buildpack should both
        // provide and require node so that it may be used without other
        // buildpacks.
        if app_dir.join("package.json").exists()
            || app_dir.join("index.js").exists()
            || app_dir.join("server.js").exists()
        {
            return pass_result
                .build_plan(plan_builder.requires("node").build())
                .build();
        }

        // This buildpack may provide node when required by other buildpacks
        // even if no common Node.js artifacts are detected. If no other
        // buildpacks require node, detection will fail.
        pass_result.build_plan(plan_builder.build()).build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header("Heroku Node.js Engine Buildpack");
        log_header("Checking Node.js version");

        let inv: Inventory =
            toml::from_str(INVENTORY).map_err(NodeJsEngineBuildpackError::InventoryParseError)?;

        let pjson_path = context.app_dir.join("package.json");
        let pjson =
            PackageJson::read(pjson_path).map_err(NodeJsEngineBuildpackError::PackageJsonError)?;
        let version_range = pjson
            .engines
            .and_then(|e| e.node)
            .unwrap_or_else(Requirement::any);
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
        let dist_layer = DistLayer {
            release: target_release.clone(),
        };
        context.handle_layer(layer_name!("dist"), dist_layer)?;

        let web_env_layer = WebEnvLayer {};
        context.handle_layer(layer_name!("web_env"), web_env_layer)?;

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
