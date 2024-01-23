use crate::layers::{
    DistLayer, DistLayerError, NodeRuntimeMetricsError, NodeRuntimeMetricsLayer, WebEnvLayer,
};
use commons::output::build_log::{BuildLog, Logger};
use heroku_nodejs_utils::errors::on_get_node_version_error;
use heroku_nodejs_utils::inv::Inventory;
use heroku_nodejs_utils::node;
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
#[cfg(test)]
use libcnb_test as _;
use libherokubuildpack::log::{log_error, log_header, log_info};
#[cfg(test)]
use serde_json as _;
use std::io::stdout;
#[cfg(test)]
use test_support as _;
use thiserror::Error;
#[cfg(test)]
use ureq as _;

mod layers;

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

        let inventory: Inventory =
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

        let target_release = inventory.resolve(&version_range).ok_or(
            NodeJsEngineBuildpackError::UnknownVersionError(version_range.to_string()),
        )?;

        log_info(format!(
            "Resolved Node.js version: {}",
            target_release.version
        ));

        log_header("Installing Node.js distribution");
        let dist_layer = context.handle_layer(
            layer_name!("dist"),
            DistLayer {
                release: target_release.clone(),
            },
        )?;

        let env = dist_layer.env.apply(Scope::Build, &Env::from_current());
        let node_version = node::get_node_version(&env)
            .map_err(NodeJsEngineBuildpackError::GetNodeVersionError)?;
        if Requirement::parse(MINIMUM_NODE_VERSION_FOR_METRICS)
            .expect("should be a valid version range")
            .satisfies(&node_version)
        {
            context.handle_layer(layer_name!("node_runtime_metrics"), NodeRuntimeMetricsLayer)?;
        }

        context.handle_layer(layer_name!("web_env"), WebEnvLayer)?;

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
                    NodeJsEngineBuildpackError::GetNodeVersionError(e) => {
                        on_get_node_version_error(
                            e,
                            BuildLog::new(stdout()).without_buildpack_name(),
                        );
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
    #[error("Couldn't get Node.js version")]
    GetNodeVersionError(node::GetNodeVersionError),
}

impl From<NodeJsEngineBuildpackError> for libcnb::Error<NodeJsEngineBuildpackError> {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsEngineBuildpack);
