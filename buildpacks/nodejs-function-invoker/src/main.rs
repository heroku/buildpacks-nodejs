#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use crate::layers::{RuntimeLayer, RuntimeLayerError};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{Launch, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, read_toml_file, Buildpack};
use libherokubuildpack::toml_select_value;
use libherokubuildpack::{log_error, log_header, log_info};
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;
use toml::Value;

mod layers;

pub struct NodeJsInvokerBuildpack;

#[derive(Deserialize, Debug)]
pub struct NodeJsInvokerBuildpackMetadata {
    pub runtime: NodeJsInvokerBuildpackRuntimeMetadata,
}

#[derive(Deserialize, Debug)]
pub struct NodeJsInvokerBuildpackRuntimeMetadata {
    pub package: String,
}

impl Buildpack for NodeJsInvokerBuildpack {
    type Platform = GenericPlatform;
    type Metadata = NodeJsInvokerBuildpackMetadata;
    type Error = NodeJsInvokerBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        detect_function(&context.app_dir)
            .then(|| {
                DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .requires("node")
                            .requires("nodejs-function-invoker")
                            .provides("nodejs-function-invoker")
                            .build(),
                    )
                    .build()
            })
            .unwrap_or_else(|| DetectResultBuilder::fail().build())
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header("Heroku Node.js Function Invoker Buildpack");
        context.handle_layer(
            layer_name!("runtime"),
            RuntimeLayer {
                package: context
                    .buildpack_descriptor
                    .metadata
                    .runtime
                    .package
                    .clone(),
            },
        )?;
        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) -> i32 {
        match error {
            libcnb::Error::BuildpackError(bp_err) => {
                let err_string = bp_err.to_string();
                match bp_err {
                    NodeJsInvokerBuildpackError::RuntimeLayerError(_) => {
                        log_error("Node.js Function Invoker Runtime error", err_string);
                        70
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
pub enum NodeJsInvokerBuildpackError {
    #[error("{0}")]
    RuntimeLayerError(#[from] RuntimeLayerError),
}

fn detect_function(dir: &PathBuf) -> bool {
    dir.join("function.toml").exists() || {
        read_toml_file(dir.join("project.toml"))
            .ok()
            .and_then(|toml: Value| {
                toml_select_value(vec!["com", "salesforce", "type"], &toml)
                    .and_then(toml::Value::as_str)
                    .map(|value| value == "function")
            })
            .unwrap_or(false)
    }
}

buildpack_main!(NodeJsInvokerBuildpack);
