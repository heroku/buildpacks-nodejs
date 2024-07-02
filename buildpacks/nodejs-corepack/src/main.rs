use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError, PackageManager};
use heroku_nodejs_utils::telemetry::init_tracer;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::layer_name;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::GenericMetadata;
use libcnb::generic::GenericPlatform;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb::{buildpack_main, Buildpack, Env};
use libherokubuildpack::log::{log_header, log_info};
use opentelemetry::trace::{TraceContextExt, Tracer};
use opentelemetry::KeyValue;

use heroku_nodejs_utils::vrs::Version;
#[cfg(test)]
use libcnb_test as _;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use test_support as _;
#[cfg(test)]
use ureq as _;

mod cfg;
mod cmd;
mod errors;

buildpack_main!(CorepackBuildpack);

struct CorepackBuildpack;

impl Buildpack for CorepackBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = CorepackBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let tracer = init_tracer(context.buildpack_descriptor.buildpack.id.to_string());
        tracer.in_span("nodejs-corepack-detect", |_cx| {
            // Corepack requires the `packageManager` key from `package.json`.
            // This buildpack won't be detected without it.
            let pkg_json_path = context.app_dir.join("package.json");
            if pkg_json_path.exists() {
                let pkg_json = PackageJson::read(pkg_json_path)
                    .map_err(CorepackBuildpackError::PackageJson)?;
                cfg::get_supported_package_manager(&pkg_json).map_or_else(
                    || DetectResultBuilder::fail().build(),
                    |pkg_mgr| {
                        DetectResultBuilder::pass()
                            .build_plan(
                                BuildPlanBuilder::new()
                                    .requires("node")
                                    .requires(&pkg_mgr)
                                    .provides(pkg_mgr)
                                    .build(),
                            )
                            .build()
                    },
                )
            } else {
                DetectResultBuilder::fail().build()
            }
        })
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let tracer = init_tracer(context.buildpack_descriptor.buildpack.id.to_string());
        tracer.in_span("nodejs-corepack-build", |cx| {
            let pkg_mgr = PackageJson::read(context.app_dir.join("package.json"))
                .map_err(CorepackBuildpackError::PackageJson)?
                .package_manager
                .ok_or(CorepackBuildpackError::PackageManagerMissing)?;

            cx.span().set_attributes([
                KeyValue::new("package_manager.name", pkg_mgr.name.clone()),
                KeyValue::new("package_manager.version", pkg_mgr.version.to_string()),
            ]);

            let env = &Env::from_current();

            let corepack_version =
                cmd::corepack_version(env).map_err(CorepackBuildpackError::CorepackVersion)?;

            cx.span().set_attribute(KeyValue::new(
                "corepack.version",
                corepack_version.to_string(),
            ));

            log_header(format!(
                "Installing {} {} via corepack {corepack_version}",
                pkg_mgr.name, pkg_mgr.version
            ));

            enable_corepack(&context, &corepack_version, &pkg_mgr, &env)?;
            prepare_corepack(&context, &pkg_mgr, &env)?;

            BuildResultBuilder::new().build()
        })
    }

    fn on_error(&self, err: libcnb::Error<Self::Error>) {
        errors::on_error(err);
    }
}

#[derive(Debug)]
enum CorepackBuildpackError {
    PackageManagerMissing,
    PackageJson(PackageJsonError),
    ShimLayer(std::io::Error),
    ManagerLayer(std::io::Error),
    CorepackVersion(cmd::Error),
    CorepackEnable(cmd::Error),
    CorepackPrepare(cmd::Error),
}

impl From<CorepackBuildpackError> for libcnb::Error<CorepackBuildpackError> {
    fn from(e: CorepackBuildpackError) -> Self {
        libcnb::Error::BuildpackError(e)
    }
}

fn enable_corepack(
    context: &BuildContext<CorepackBuildpack>,
    corepack_version: &Version,
    package_manager: &PackageManager,
    env: &Env,
) -> Result<(), libcnb::Error<CorepackBuildpackError>> {
    let new_metadata = ShimLayerMetadata {
        corepack_version: corepack_version.clone(),
        layer_version: SHIM_LAYER_VERSION.to_string(),
    };

    let shim_layer = context.cached_layer(
        layer_name!("shim"),
        CachedLayerDefinition {
            launch: true,
            build: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &ShimLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match shim_layer.state {
        LayerState::Restored { .. } => {
            log_info("Restoring corepack shim cache");
        }
        LayerState::Empty { .. } => {
            log_info("Corepack change detected. Clearing corepack shim cache");
            shim_layer.write_metadata(new_metadata)?;
            std::fs::create_dir(shim_layer.path().join("bin"))
                .map_err(CorepackBuildpackError::ShimLayer)?;
        }
    }

    cmd::corepack_enable(&package_manager.name, &shim_layer.path().join("bin"), env)
        .map_err(CorepackBuildpackError::CorepackEnable)?;

    Ok(())
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ShimLayerMetadata {
    corepack_version: Version,
    layer_version: String,
}

const SHIM_LAYER_VERSION: &str = "1";

fn prepare_corepack(
    context: &BuildContext<CorepackBuildpack>,
    package_manager: &PackageManager,
    env: &Env,
) -> Result<(), libcnb::Error<CorepackBuildpackError>> {
    let new_metadata = ManagerLayerMetadata {
        manager_name: package_manager.name.clone(),
        manager_version: package_manager.version.clone(),
        layer_version: MANAGER_LAYER_VERSION.to_string(),
    };

    let manager_layer = context.cached_layer(
        layer_name!("mgr"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &ManagerLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match manager_layer.state {
        LayerState::Restored { .. } => {
            log_info("Restoring corepack package manager cache");
        }
        LayerState::Empty { .. } => {
            log_info("Package manager change detected. Clearing corepack package manager cache");
            manager_layer.write_metadata(new_metadata)?;
            let cache_path = manager_layer.path().join("cache");
            std::fs::create_dir(&cache_path).map_err(CorepackBuildpackError::ManagerLayer)?;
            manager_layer.write_env(LayerEnv::new().chainable_insert(
                Scope::All,
                ModificationBehavior::Override,
                "COREPACK_HOME",
                cache_path,
            ))?
        }
    }

    let mgr_env = manager_layer
        .read_env()
        .map(|layer_env| layer_env.apply(Scope::Build, env))?;

    cmd::corepack_prepare(&mgr_env).map_err(CorepackBuildpackError::CorepackPrepare)?;

    Ok(())
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagerLayerMetadata {
    manager_name: String,
    manager_version: Version,
    layer_version: String,
}

const MANAGER_LAYER_VERSION: &str = "1";
