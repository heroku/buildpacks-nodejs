use std::path::Path;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::launch::{Launch, ProcessBuilder};
use libcnb::data::build_plan::{BuildPlanBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericPlatform;
use libcnb::generic::GenericMetadata;
use libcnb::{buildpack_main, Buildpack};
use libcnb_nodejs::versions::{Inventory,Req};
use libcnb_nodejs::package_json::{PackageJson,PackageJsonError};
use libherokubuildpack::DownloadError;

use crate::layers::{DistLayer, WebEnvLayer};

mod layers;

const INVENTORY: &str = include_str!("../inventory.toml");

pub struct NodeJsRuntimeBuildpack;

impl Buildpack for NodeJsRuntimeBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodeJsBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if !context.app_dir.join("package.json").exists() {
            return DetectResultBuilder::fail().build();
        }
        DetectResultBuilder::pass().build_plan(
            BuildPlanBuilder::new()
                .provides("node")
                .requires("node")
                .build()
        ).build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let inv: Inventory = toml::from_str(INVENTORY).map_err(NodeJsBuildpackError::InventoryParseError)?;

        println!("---> Node.js Runtime Buildpack");
        println!("---> Checking Node.js version");
        let pjson_path = context.app_dir.join("package.json");
        let pjson = PackageJson::read(pjson_path).map_err(NodeJsBuildpackError::PackageJsonError)?;
        let node_version_range = pjson.engines
            .and_then(|e| e.node)
            .unwrap_or(Req::any());
        println!("---> Detected Node.js version range: {}", node_version_range.to_string());
        let target_release = inv.resolve(node_version_range)
            .ok_or(NodeJsBuildpackError::UnknownVersionError())?;
        println!("---> Resolved Node.js version: {}", target_release.version);

        let dist_layer = DistLayer{release: target_release.clone()};
        context.handle_layer(layer_name!("dist"), dist_layer)?;

        let web_env_layer = WebEnvLayer{};
        context.handle_layer(layer_name!("web_env"), web_env_layer)?;

        let launchjs = ["server.js", "index.js"]
            .map(|f| Path::new(&context.app_dir).join(f))
            .iter()
            .find(|p| p.exists())
            .and_then(|p| p.to_str())
            .and_then(|p| Some(
                    Launch::new().process(
                        ProcessBuilder::new(process_type!("web"), "node")
                            .args(vec![p])
                            .default(true)
                            .build()
                    )
                )
            );


        let resulter = BuildResultBuilder::new();
        match launchjs {
            Some(l) => resulter.launch(l).build(),
            None => resulter.build()
        }
    }
}

#[derive(Debug)]
pub enum NodeJsBuildpackError {
    InventoryParseError(toml::de::Error),
    PackageJsonError(PackageJsonError),
    UnknownVersionError(),
    DownloadError(DownloadError),
    ReadBinError(),
    UntarError(std::io::Error),
    CreateTempFileError(std::io::Error),
}

impl From<NodeJsBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(e: NodeJsBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsRuntimeBuildpack);
