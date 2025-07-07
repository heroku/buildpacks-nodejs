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
use libcnb::data::buildpack_plan::BuildpackPlan;
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
use std::path::Path;
use std::str::FromStr;
#[cfg(test)]
use test_support as _;
use toml_edit::{DocumentMut, TableLike};

mod configure_available_parallelism;
mod configure_web_env;
mod errors;
mod install_node;

const INVENTORY: &str = include_str!("../inventory.toml");

const LTS_VERSION: &str = "22.x";

const CONFIG_NAMESPACE: &str = "com.heroku.buildpacks.nodejs";

struct NodeJsEngineBuildpack;

impl Buildpack for NodeJsEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodeJsEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let mut plan_builder = BuildPlanBuilder::new()
            .provides("node")
            .provides("npm")
            .provides(CONFIG_NAMESPACE)
            .or()
            .provides("node")
            .provides(CONFIG_NAMESPACE);

        // If there are common node artifacts, this buildpack should both
        // provide and require node so that it may be used without other
        // buildpacks.
        if ["package.json", "index.js", "server.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .any(|path| path.exists())
            || project_toml_contains_namespaced_config(&context.app_dir)
        {
            plan_builder = plan_builder.requires("node");
        }

        plan_builder = plan_builder.requires(CONFIG_NAMESPACE);

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

        let requested_version_range =
            read_node_version_config(&context.app_dir, &context.buildpack_plan)?;

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

fn project_toml_contains_namespaced_config(app_dir: &Path) -> bool {
    if app_dir.join("project.toml").try_exists().unwrap_or(false) {
        let contents = std::fs::read_to_string(app_dir.join("project.toml"))
            .expect("Could not read project.toml file");
        let project_toml =
            toml_edit::DocumentMut::from_str(&contents).expect("Could not parse project.toml file");
        get_buildpack_namespaced_config(&project_toml).is_some()
    } else {
        false
    }
}

fn read_node_version_config(
    app_dir: &Path,
    buildpack_plan: &BuildpackPlan,
) -> Result<Option<Requirement>, NodeJsEngineBuildpackError> {
    // package.json is the where we source the requested version from first
    if app_dir.join("package.json").try_exists().unwrap_or(false) {
        let requested_version_range = PackageJson::read(app_dir.join("package.json"))
            .map_err(NodeJsEngineBuildpackError::PackageJson)
            .map(|package_json| package_json.engines.and_then(|e| e.node))?;

        if requested_version_range.is_some() {
            return Ok(requested_version_range);
        }
    }

    // if that's not available, then we can use project.toml next
    if app_dir.join("project.toml").try_exists().unwrap_or(false) {
        let contents = std::fs::read_to_string(app_dir.join("project.toml"))
            .expect("Could not read project.toml file");
        let project_toml =
            toml_edit::DocumentMut::from_str(&contents).expect("Could not parse project.toml file");
        if let Some(config) = get_buildpack_namespaced_config(&project_toml) {
            return Ok(get_runtime_version(config));
        }
    }

    // finally, lowest-priority is whatever buildpack plans have been contributed
    let buildpack_plan_configs = buildpack_plan
        .entries
        .iter()
        .filter(|entry| entry.name == CONFIG_NAMESPACE);

    for buildpack_plan_config in buildpack_plan_configs {
        let toml = toml::to_string(&buildpack_plan_config.metadata)
            .expect("Buildplan metadata should be serializable to TOML");
        let doc = toml_edit::DocumentMut::from_str(&toml)
            .expect("Buildplan metadata should be deserializable from TOML");
        let config = doc
            .as_item()
            .as_table_like()
            .expect("Buildplan metadata should always be a table");
        if let Some(requested_version) = get_runtime_version(config) {
            return Ok(Some(requested_version));
        }
    }

    Ok(None)
}

fn get_buildpack_namespaced_config(doc: &DocumentMut) -> Option<&dyn TableLike> {
    let mut current_table = doc
        .as_item()
        .as_table_like()
        .expect("The 'project.toml' contents should always be a table");
    for name in CONFIG_NAMESPACE.split('.') {
        current_table = match current_table.get(name) {
            Some(item) => item
                .as_table_like()
                .expect("Error parsing namespaced config in project.toml at {name}"),
            None => return None,
        };
    }
    Some(current_table)
}

fn get_runtime_version(namespaced_config: &dyn TableLike) -> Option<Requirement> {
    if let Some(runtime) = namespaced_config
        .get("runtime")
        .and_then(|value| value.as_table_like())
    {
        runtime
            .get("version")
            .and_then(|value| value.as_str())
            .map(|version| Requirement::parse(version).expect("Could not parse runtime version"))
    } else {
        None
    }
}
