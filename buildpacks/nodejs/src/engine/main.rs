// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::engine::configure_available_parallelism::configure_available_parallelism;
use crate::engine::configure_web_env::configure_web_env;
use crate::engine::errors;
use crate::engine::install_node::{install_node, DistLayerError};
use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::Env;
use libherokubuildpack::inventory::artifact::{Arch, Os};
use libherokubuildpack::inventory::Inventory;
use sha2::Sha256;
use std::env::consts;

const INVENTORY: &str = include_str!("../../../../inventory/nodejs.toml");

const LTS_VERSION: &str = "22.x";

pub(crate) fn build(
    context: &BuildContext<NodeJsBuildpack>,
    mut env: Env,
    mut build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodeJsBuildpackError> {
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

    env = install_node(context, target_artifact, env)?;
    env = configure_available_parallelism(context, env)?;
    configure_web_env(context)?;

    let launchjs = ["server.js", "index.js"]
        .map(|name| context.app_dir.join(name))
        .iter()
        .find(|path| path.exists())
        .map(|path| {
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), ["node", &path.to_string_lossy()])
                        .default(true)
                        .build(),
                )
                .build()
        });

    if let Some(launchjs) = launchjs {
        build_result_builder = build_result_builder.launch(launchjs);
    }

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NodeJsEngineBuildpackError) {
    print::plain(errors::on_nodejs_engine_buildpack_error(error).to_string());
}

#[derive(Debug)]
pub(crate) enum NodeJsEngineBuildpackError {
    InventoryParse(toml::de::Error),
    PackageJson(PackageJsonError),
    UnknownVersion(String),
    DistLayer(DistLayerError),
}

impl From<NodeJsEngineBuildpackError> for libcnb::Error<NodeJsBuildpackError> {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodeJsBuildpackError::NodeEngine(e))
    }
}
