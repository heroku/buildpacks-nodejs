#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use crate::function::{get_main, is_function, MainError};
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

mod function;
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
        is_function(&context.app_dir)
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

        get_main(&context.app_dir).map_err(NodeJsInvokerBuildpackError::MainFunctionError)?;

        BuildResultBuilder::new()
            .launch(
                Launch::new().process(
                    ProcessBuilder::new(process_type!("web"), "sf-fx-runtime-nodejs")
                        .args(vec![
                            "serve",
                            &context.app_dir.to_string_lossy(),
                            "--workers",
                            "2",
                            "--host",
                            "::",
                            "--port",
                            "${PORT:-8080}",
                            "--debug-port",
                            "${DEBUG_PORT:-}",
                        ])
                        .default(true)
                        .build(),
                ),
            )
            .build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) -> i32 {
        match error {
            libcnb::Error::BuildpackError(bp_err) => {
                let err_string = bp_err.to_string();
                match bp_err {
                    NodeJsInvokerBuildpackError::MainFunctionError(_) => {
                        log_error(
                            "Node.js Function Invoker main function detection error",
                            err_string,
                        );
                        70
                    }
                    NodeJsInvokerBuildpackError::RuntimeLayerError(_) => {
                        log_error("Node.js Function Invoker runtime layer error", err_string);
                        71
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
    MainFunctionError(#[from] MainError),
    #[error("{0}")]
    RuntimeLayerError(#[from] RuntimeLayerError),
}

impl From<NodeJsInvokerBuildpackError> for libcnb::Error<NodeJsInvokerBuildpackError> {
    fn from(e: NodeJsInvokerBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsInvokerBuildpack);
