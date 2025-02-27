use crate::attach_startup_script::{
    attach_startup_script, ScriptLayerError, NODEJS_RUNTIME_SCRIPT,
};
use crate::function::{
    get_declared_runtime_package_version, get_main, is_function, ExplicitRuntimeDependencyError,
    MainError,
};
use crate::install_nodejs_function_runtime::{install_nodejs_function_runtime, RuntimeLayerError};
#[cfg(test)]
use base64 as _;
use bullet_stream::Print;
#[cfg(test)]
use hex as _;
use indoc::formatdoc;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericPlatform;
use libcnb::{buildpack_main, Buildpack, Error};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use rand as _;
use serde::Deserialize;
use std::io::stderr;
#[cfg(test)]
use test_support as _;
use thiserror::Error;
#[cfg(test)]
use ureq as _;

mod attach_startup_script;
mod function;
mod install_nodejs_function_runtime;

struct NodeJsInvokerBuildpack;

#[derive(Deserialize, Debug)]
struct NodeJsInvokerBuildpackMetadata {
    runtime: NodeJsInvokerBuildpackRuntimeMetadata,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct NodeJsInvokerBuildpackRuntimeMetadata {
    package_name: String,
    package_version: String,
}

impl Buildpack for NodeJsInvokerBuildpack {
    type Platform = GenericPlatform;
    type Metadata = NodeJsInvokerBuildpackMetadata;
    type Error = NodeJsInvokerBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        is_function(context.app_dir)
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
        let mut log = Print::new(stderr()).h1(context
            .buildpack_descriptor
            .buildpack
            .name
            .as_ref()
            .expect("The buildpack should have a name"));

        let app_dir = &context.app_dir;
        let metadata_runtime = &context.buildpack_descriptor.metadata.runtime;
        let package_name = &metadata_runtime.package_name;
        let package_version = &metadata_runtime.package_version;

        log = log.bullet("Checking for function file").done();
        get_main(app_dir).map_err(NodeJsInvokerBuildpackError::MainFunction)?;

        let declared_runtime_package_version =
            get_declared_runtime_package_version(app_dir, package_name)
                .map_err(NodeJsInvokerBuildpackError::ExplicitRuntimeDependencyFunction)?;

        if let Some(package_version) = declared_runtime_package_version.clone() {
            log = log
                .bullet(format!(
                    "Node.js function runtime declared in package.json: {0}@{1}",
                    package_name.clone(),
                    package_version
                ))
                .done();
        } else {
            log = log.warning(
                format!("\
                    Deprecation: Future versions of the Functions Runtime for Node.js ({package_name}) will not be auto-detected \
                    and must be added as a dependency in package.json."
                )
            );
            log = install_nodejs_function_runtime(
                &context,
                &format!("{package_name}@{package_version}"),
                log,
            )?;
        }

        let command = match declared_runtime_package_version {
            Some(_) => "node_modules/.bin/sf-fx-runtime-nodejs", // local  (explicit)
            None => "sf-fx-runtime-nodejs",                      // global (implicit)
        };

        attach_startup_script(&context)?;

        log.done();

        BuildResultBuilder::new()
            .launch(
                LaunchBuilder::new()
                    .process(
                        ProcessBuilder::new(
                            process_type!("web"),
                            [
                                NODEJS_RUNTIME_SCRIPT,
                                command,
                                &context.app_dir.to_string_lossy(),
                            ],
                        )
                        .default(true)
                        .build(),
                    )
                    .build(),
            )
            .build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        let log = Print::new(stderr()).without_header();
        match error {
            Error::BuildpackError(buildpack_error) => match buildpack_error {
                NodeJsInvokerBuildpackError::MainFunction(e) => {
                    log.error(formatdoc! { "
                            Node.js Function Invoker main function detection error

                            {e}
                        "});
                }
                NodeJsInvokerBuildpackError::RuntimeLayer(e) => {
                    log.error(formatdoc! { "
                            Node.js Function Invoker runtime layer error

                            {e}
                        "});
                }
                NodeJsInvokerBuildpackError::ScriptLayer(e) => {
                    log.error(formatdoc! { "
                            Node.js Function Invoker script layer error

                            {e}
                        "});
                }
                NodeJsInvokerBuildpackError::ExplicitRuntimeDependencyFunction(e) => {
                    log.error(formatdoc! { "
                            Node.js Function Invoker explicit Node.js function runtime dependency error

                            {e}
                        "});
                }
            },
            framework_error => {
                log.error(formatdoc! {"
                    heroku/nodejs-function-invoker internal buildpack error
                    
                    An unexpected internal error was reported by the framework used \
                    by this buildpack.
        
                    If the issue persists, consider opening an issue on the GitHub \
                    repository. If you are unable to deploy to Heroku as a result \
                    of this issue, consider opening a ticket for additional support.
        
                    Details: {framework_error}
                "});
            }
        };
    }
}

#[derive(Error, Debug)]
enum NodeJsInvokerBuildpackError {
    #[error("{0}")]
    MainFunction(#[from] MainError),
    #[error("{0}")]
    RuntimeLayer(#[from] RuntimeLayerError),
    #[error("{0}")]
    ScriptLayer(#[from] ScriptLayerError),
    #[error("{0}")]
    ExplicitRuntimeDependencyFunction(#[from] ExplicitRuntimeDependencyError),
}

impl From<NodeJsInvokerBuildpackError> for libcnb::Error<NodeJsInvokerBuildpackError> {
    fn from(e: NodeJsInvokerBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsInvokerBuildpack);
