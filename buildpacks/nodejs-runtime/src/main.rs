use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{Launch, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericPlatform;
use libcnb::generic::GenericMetadata;
use libcnb::{buildpack_main, Buildpack};
use libcnb_nodejs::versions::{Software,Req};
use libcnb_nodejs::package_json::{PackageJson,PackageJsonError};

use crate::layers::{RuntimeLayer};
use crate::util::{DownloadError, UntarError};

mod util;
mod layers;

const INVENTORY: &str = include_str!("../inventory.toml");

pub struct NodeJsRuntimeBuildpack;

impl Buildpack for NodeJsRuntimeBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodeJsBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if context.app_dir.join("package.json").exists() {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::pass().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let software: Software = toml::from_str(INVENTORY).map_err(NodeJsBuildpackError::InventoryParseError)?;

        println!("---> Node.js Runtime Buildpack");
        println!("---> Checking Node.js version");
        let pjson_path = context.app_dir.join("package.json");
        let pjson = PackageJson::read(pjson_path).map_err(NodeJsBuildpackError::PackageJsonError)?;
        let node_version_range = pjson.engines
            .and_then(|e| e.node)
            .unwrap_or(Req::any());
        println!("---> Detected Node.js version range: {}", node_version_range.to_string());
        let target_release = software.resolve(node_version_range, "linux-x64", "release")
            .ok_or(NodeJsBuildpackError::UnknownVersionError())?;
        println!("---> Resolved Node.js version: {}", target_release.version);

        let runtime_layer = RuntimeLayer{release: target_release.clone()};
        context.handle_layer(layer_name!("runtime"), runtime_layer)?;

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

#[derive(Debug)]
pub enum NodeJsBuildpackError {
    InventoryParseError(toml::de::Error),
    PackageJsonError(PackageJsonError),
    UnknownVersionError(),
    DownloadError(DownloadError),
    ReadBinError(),
    UntarError(UntarError),
    CreateTempFileError(std::io::Error),
}

impl From<NodeJsBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(e: NodeJsBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsRuntimeBuildpack);
