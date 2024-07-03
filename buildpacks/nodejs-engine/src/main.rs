use std::env::consts;

use crate::attach_runtime_metrics::{configure_web_env, NodeRuntimeMetricsError};
use crate::configure_web_env::attach_runtime_metrics;
use crate::install_node::{install_node, DistLayerError};
use heroku_inventory_utils::inv::{resolve, Arch, Inventory, Os};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack};
#[cfg(test)]
use libcnb_test as _;
use libherokubuildpack::log::{log_error, log_header, log_info};
#[cfg(test)]
use serde_json as _;
use sha2::Sha256;
#[cfg(test)]
use test_support as _;
use thiserror::Error;
#[cfg(test)]
use ureq as _;

mod attach_runtime_metrics;
mod configure_web_env;
mod install_node;

const INVENTORY: &str = include_str!("../inventory.toml");

const LTS_VERSION: &str = "20.x";

const MINIMUM_NODE_VERSION_FOR_METRICS: &str = ">=14.10";

struct NodeJsEngineBuildpack;

impl Buildpack for NodeJsEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodeJsEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let mut plan_builder = BuildPlanBuilder::new()
            .provides("node")
            .provides("npm")
            .or()
            .provides("node");

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

        let inv: Inventory<Version, Sha256> =
            toml::from_str(INVENTORY).map_err(NodeJsEngineBuildpackError::InventoryParseError)?;

        let requested_version_range = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(NodeJsEngineBuildpackError::PackageJsonError)
            .map(|package_json| package_json.engines.and_then(|e| e.node))?;

        let version_range = if let Some(value) = requested_version_range {
            log_info(format!("Detected Node.js version range: {value}"));
            value
        } else {
            log_info(format!(
                "Node.js version not specified, using {LTS_VERSION}"
            ));
            Requirement::parse(LTS_VERSION).expect("The default Node.js version should be valid")
        };

        let target_artifact = match (consts::OS.parse::<Os>(), consts::ARCH.parse::<Arch>()) {
            (Ok(os), Ok(arch)) => resolve(&inv.artifacts, os, arch, &version_range),
            (_, _) => None,
        }
        .ok_or(NodeJsEngineBuildpackError::UnknownVersionError(
            version_range.to_string(),
        ))?;

        log_info(format!(
            "Resolved Node.js version: {}",
            target_artifact.version
        ));

        log_header("Installing Node.js distribution");
        install_node(&context, target_artifact)?;

        configure_web_env(&context)?;

        if Requirement::parse(MINIMUM_NODE_VERSION_FOR_METRICS)
            .expect("should be a valid version range")
            .satisfies(&target_artifact.version)
        {
            attach_runtime_metrics(&context)?;
        }

        let launchjs = ["server.js", "index.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .find(|path| path.exists())
            .map(|path| {
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(
                            process_type!("web"),
                            ["node", &path.to_string_lossy()],
                        )
                        .default(true)
                        .build(),
                    )
                    .build()
            });

        let resulter = BuildResultBuilder::new();
        match launchjs {
            Some(l) => resulter.launch(l).build(),
            None => resulter.build(),
        }
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        match error {
            libcnb::Error::BuildpackError(bp_err) => {
                let err_string = bp_err.to_string();
                match bp_err {
                    NodeJsEngineBuildpackError::DistLayerError(_) => {
                        log_error("Node.js engine distribution error", err_string);
                    }
                    NodeJsEngineBuildpackError::InventoryParseError(_) => {
                        log_error("Node.js engine inventory parse error", err_string);
                    }
                    NodeJsEngineBuildpackError::PackageJsonError(_) => {
                        log_error("Node.js engine package.json error", err_string);
                    }
                    NodeJsEngineBuildpackError::UnknownVersionError(_) => {
                        log_error("Node.js engine version error", err_string);
                    }
                    NodeJsEngineBuildpackError::NodeRuntimeMetricsError(_) => {
                        log_error("Node.js engine runtime metric error", err_string);
                    }
                }
            }
            err => {
                log_error("Internal Buildpack Error", err.to_string());
            }
        };
    }
}

#[derive(Error, Debug)]
enum NodeJsEngineBuildpackError {
    #[error("Couldn't parse Node.js inventory: {0}")]
    InventoryParseError(toml::de::Error),
    #[error("Couldn't parse package.json: {0}")]
    PackageJsonError(PackageJsonError),
    #[error("Couldn't resolve Node.js version: {0}")]
    UnknownVersionError(String),
    #[error(transparent)]
    DistLayerError(#[from] DistLayerError),
    #[error(transparent)]
    NodeRuntimeMetricsError(#[from] NodeRuntimeMetricsError),
}

impl From<NodeJsEngineBuildpackError> for libcnb::Error<NodeJsEngineBuildpackError> {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsEngineBuildpack);
