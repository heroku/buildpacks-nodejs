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
        let inv: Inventory = toml::from_str(INVENTORY).map_err(NodeJsEngineBuildpackError::InventoryParseError)?;

        println!("---> Node.js Runtime Buildpack");
        println!("---> Checking Node.js version");
        let pjson_path = context.app_dir.join("package.json");
        let pjson = PackageJson::read(pjson_path).map_err(NodeJsEngineBuildpackError::PackageJsonError)?;
        let version_range = pjson.engines
            .and_then(|e| e.node)
            .unwrap_or(Req::any());
        let version_range_string = version_range.to_string();
        println!("---> Detected Node.js version range: {}", version_range_string);
        let target_release = inv.resolve(version_range)
            .ok_or(NodeJsEngineBuildpackError::UnknownVersionError(version_range_string))?;
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

#[derive(Error, Debug)]
pub enum NodeJsEngineBuildpackError {
    #[error("Couldn't parse Node.js inventory: {0}")]
    InventoryParseError(toml::de::Error),
    #[error("Couldn't parse package.json: {0}")]
    PackageJsonError(PackageJsonError),
    #[error("Couldn't resolve Node.js version: {0}")]
    UnknownVersionError(String),
    #[error("dist layer error: {0}")]
    DistLayerError(#[from]DistLayerError),
}

impl From<NodeJsEngineBuildpackError> for libcnb::Error<NodeJsEngineBuildpackError> {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsEngineBuildpack);
