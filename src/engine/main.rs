// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use super::install_node::{DistLayerError, install_node};
use crate::utils::error_handling::ErrorMessage;
use crate::utils::package_json::{PackageJson, PackageJsonError};
use crate::utils::vrs::{Requirement, Version};
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use libcnb::Env;
use libherokubuildpack::inventory::Inventory;
use libherokubuildpack::inventory::artifact::{Arch, Os};
use sha2::Sha256;
use std::env::consts;

const INVENTORY: &str = include_str!("../../inventory/nodejs.toml");

const LTS_VERSION: &str = "22.x";

pub(crate) fn build(context: &BuildpackBuildContext, mut env: Env) -> BuildpackResult<Env> {
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

    env = install_node(context, &env, target_artifact)?;

    Ok(env)
}

pub(crate) fn on_error(error: NodeJsEngineBuildpackError) -> ErrorMessage {
    super::errors::on_nodejs_engine_error(error)
}

#[derive(Debug)]
pub(crate) enum NodeJsEngineBuildpackError {
    InventoryParse(toml::de::Error),
    PackageJson(PackageJsonError),
    UnknownVersion(String),
    DistLayer(DistLayerError),
}

impl From<NodeJsEngineBuildpackError> for BuildpackError {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        NodeJsBuildpackError::NodeEngine(e).into()
    }
}
