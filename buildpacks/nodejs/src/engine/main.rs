// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::configure_available_parallelism::configure_available_parallelism;
use crate::configure_web_env::configure_web_env;
use crate::install_node::{install_node, DistLayerError};
use bullet_stream::global::print;
use bullet_stream::style;
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
use libherokubuildpack::inventory::artifact::{Arch, Os};
use libherokubuildpack::inventory::Inventory;
#[cfg(test)]
use regex as _;
#[cfg(test)]
use serde_json as _;
use sha2::Sha256;
use std::env::consts;
#[cfg(test)]
use test_support as _;

mod configure_available_parallelism;
mod configure_web_env;
mod errors;
mod install_node;

const INVENTORY: &str = include_str!("../inventory.toml");

const LTS_VERSION: &str = "22.x";

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
        let buildpack_start = print::buildpack(
            context
                .buildpack_descriptor
                .buildpack
                .name
                .as_ref()
                .expect("The buildpack.toml should have a 'name' field set"),
        );

        print::bullet("Checking Node.js version");

        let inv: Inventory<Version, Sha256, Option<()>> =
            toml::from_str(INVENTORY).map_err(NodeJsEngineBuildpackError::InventoryParse)?;

        let requested_version_range = PackageJson::read(context.app_dir.join("package.json"))
            .map_err(NodeJsEngineBuildpackError::PackageJson)
            .map(|package_json| package_json.engines.and_then(|e| e.node))?;

        let version_range = if let Some(value) = requested_version_range {
            print::sub_bullet(format!(
                "Detected Node.js version range: {}",
                style::value(value.to_string())
            ));
            value
        } else {
            print::sub_bullet(format!(
                "Node.js version not specified, using {}",
                style::value(LTS_VERSION)
            ));
            Requirement::parse(LTS_VERSION).expect("The default Node.js version should be valid")
        };

        let target_artifact = match (consts::OS.parse::<Os>(), consts::ARCH.parse::<Arch>()) {
            (Ok(os), Ok(arch)) => inv.resolve(os, arch, &version_range),
            (_, _) => None,
        }
        .ok_or(NodeJsEngineBuildpackError::UnknownVersion(
            version_range.to_string(),
        ))?;

        print::sub_bullet(format!(
            "Resolved Node.js version: {}",
            style::value(target_artifact.version.to_string())
        ));

        install_node(&context, target_artifact)?;
        configure_web_env(&context)?;
        configure_available_parallelism(&context)?;

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

        print::all_done(&Some(buildpack_start));

        let resulter = BuildResultBuilder::new();
        match launchjs {
            Some(l) => resulter.launch(l).build(),
            None => resulter.build(),
        }
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        let error_message = errors::on_error(error);
        eprintln!("\n{error_message}");
    }
}

#[derive(Debug)]
enum NodeJsEngineBuildpackError {
    InventoryParse(toml::de::Error),
    PackageJson(PackageJsonError),
    UnknownVersion(String),
    DistLayer(DistLayerError),
}

impl From<NodeJsEngineBuildpackError> for libcnb::Error<NodeJsEngineBuildpackError> {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

buildpack_main!(NodeJsEngineBuildpack);
