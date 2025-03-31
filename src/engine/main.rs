use crate::common::application;
use crate::common::package_json::{PackageJson, PackageJsonError};
use crate::common::vrs::{Requirement, Version};
use crate::engine::configure_web_env::configure_web_env;
use crate::engine::errors;
use crate::engine::install_node::{install_node, DistLayerError};
use crate::{NodejsBuildpack, NodejsBuildpackError};
use bullet_stream::{style, Print};
use libcnb::build::{BuildContext, BuildResultBuilder};
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use libcnb::Env;
use libherokubuildpack::inventory::artifact::{Arch, Os};
use libherokubuildpack::inventory::Inventory;
use sha2::Sha256;
use std::env::consts;
use std::io::stderr;

const INVENTORY: &str = include_str!("../../inventory/node.toml");

const LTS_VERSION: &str = "22.x";

pub(crate) fn build(
    context: &BuildContext<NodejsBuildpack>,
    mut env: Env,
    mut build_result_builder: BuildResultBuilder,
) -> libcnb::Result<(Env, BuildResultBuilder), NodejsBuildpackError> {
    let mut log = Print::new(stderr()).h1(context
        .buildpack_descriptor
        .buildpack
        .name
        .as_ref()
        .expect("The buildpack.toml should have a 'name' field set"));

    let package_json = PackageJson::read(context.app_dir.join("package.json"))
        .map_err(NodeJsEngineBuildpackError::PackageJson)?;

    if package_json.has_dependencies() {
        application::check_for_singular_lockfile(&context.app_dir)
            .map_err(NodeJsEngineBuildpackError::Application)?;
    }

    let mut bullet = log.bullet("Checking Node.js version");

    let inv: Inventory<Version, Sha256, Option<()>> =
        toml::from_str(INVENTORY).map_err(NodeJsEngineBuildpackError::InventoryParse)?;

    let requested_version_range = package_json.engines.and_then(|e| e.node);

    let version_range = if let Some(value) = requested_version_range {
        bullet = bullet.sub_bullet(format!(
            "Detected Node.js version range: {}",
            style::value(value.to_string())
        ));
        value
    } else {
        bullet = bullet.sub_bullet(format!(
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

    log = bullet
        .sub_bullet(format!(
            "Resolved Node.js version: {}",
            style::value(target_artifact.version.to_string())
        ))
        .done();

    (env, log) = install_node(&context, env, target_artifact, log)?;

    configure_web_env(&context)?;

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

    log.done();

    if let Some(launchjs) = launchjs {
        build_result_builder = build_result_builder.launch(launchjs);
    }

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NodeJsEngineBuildpackError) {
    let error_message = errors::on_error(error);
    eprintln!("\n{error_message}");
}

#[derive(Debug)]
pub(crate) enum NodeJsEngineBuildpackError {
    InventoryParse(toml::de::Error),
    PackageJson(PackageJsonError),
    UnknownVersion(String),
    DistLayer(DistLayerError),
    Application(application::Error),
}

impl From<NodeJsEngineBuildpackError> for libcnb::Error<NodejsBuildpackError> {
    fn from(e: NodeJsEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(NodejsBuildpackError::NodeEngine(e))
    }
}
