#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]

use crate::function::{get_declared_runtime_package, get_main, is_function, MainError};
use crate::layers::{RuntimeLayer, RuntimeLayerError};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack};
#[cfg(test)]
use libcnb_test as _;
use libherokubuildpack::error::on_error;
use libherokubuildpack::log::{log_error, log_header, log_info, log_warning};
use serde::Deserialize;
#[cfg(test)]
use test_support as _;
use thiserror::Error;
#[cfg(test)]
use ureq as _;

mod function;
mod layers;

pub struct NodeJsInvokerBuildpack;

#[derive(Deserialize, Debug)]
pub struct NodeJsInvokerBuildpackMetadata {
    pub runtime: NodeJsInvokerBuildpackRuntimeMetadata,
}

#[derive(Deserialize, Debug)]
pub struct NodeJsInvokerBuildpackRuntimeMetadata {
    pub package_name: String,
    pub package_version: String,
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

        log_info("Checking for function file");
        get_main(&context.app_dir).map_err(NodeJsInvokerBuildpackError::MainFunctionError)?;

        if let Some((package_name, package_version)) = get_declared_runtime_package(&context) {
            log_info(format!(
                "Runtime declared in package.json: {0}@{1}",
                package_name.clone(),
                package_version
            ));
        } else {
            let package_name = context
                .buildpack_descriptor
                .metadata
                .runtime
                .package_name
                .clone();
            let package_version = context
                .buildpack_descriptor
                .metadata
                .runtime
                .package_version
                .clone();
            log_warning(
                "Deprecation",
                format!("Future versions of the Functions Runtime for Node.js ({0}) will not be auto-detected \
                and must be added as a dependency in package.json.", package_name)
            );
            context.handle_layer(
                layer_name!("implicit_runtime"),
                RuntimeLayer {
                    package: format!("{0}@{1}", package_name, package_version),
                },
            )?;
        }

        BuildResultBuilder::new()
            .launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(process_type!("web"), "npx")
                            .args(vec![
                                "sf-fx-runtime-nodejs",
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
                            .direct(false)
                            .build(),
                    )
                    .build(),
            )
            .build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        on_error(
            |bp_err| {
                let err_string = bp_err.to_string();
                match bp_err {
                    NodeJsInvokerBuildpackError::MainFunctionError(_) => {
                        log_error(
                            "Node.js Function Invoker main function detection error",
                            err_string,
                        );
                    }
                    NodeJsInvokerBuildpackError::RuntimeLayerError(_) => {
                        log_error("Node.js Function Invoker runtime layer error", err_string);
                    }
                }
            },
            error,
        );
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
