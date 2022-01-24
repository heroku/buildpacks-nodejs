use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{Launch, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericPlatform;
use libcnb::layer_env::Scope;
use libcnb::{buildpack_main, Buildpack};

use crate::layers::{RuntimeLayer};
use crate::util::{DownloadError, UntarError};
use serde::Deserialize;
use std::process::ExitStatus;

mod util;
mod layers;

pub struct NodejsRuntimeBuildpack;

impl Buildpack for NodejsRuntimeBuildpack {
    type Platform = GenericPlatform;
    type Metadata = NodejsBuildpackMetadata;
    type Error = NodejsBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if context.app_dir.join("package.json").exists() {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        println!("---> Node.js Runtime Buildpack");

        let runtime_layer = context.handle_layer(layer_name!("runtime"), RuntimeLayer)?;

        BuildResultBuilder::new()
            .launch(
                Launch::new()
                    .process(
                        ProcessBuilder::new(process_type!("web"), "npm")
                            .args(vec!["start"])
                            .default(true)
                            .build(),
                    )
            )
            .build()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct NodejsBuildpackMetadata {
    pub nodejs_runtime_url: String,
}

#[derive(Debug)]
pub enum NodejsBuildpackError {
    NodejsDownloadError(DownloadError),
    NodejsUntarError(UntarError),
    CouldNotCreateTemporaryFile(std::io::Error),
}

buildpack_main!(NodejsRuntimeBuildpack);
