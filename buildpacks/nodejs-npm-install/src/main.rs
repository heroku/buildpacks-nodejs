mod cmd;
mod errors;
mod layers;

use crate::errors::{log_user_errors, NpmInstallBuildpackError};
use crate::layers::npm_cache::NpmCacheLayer;
use heroku_nodejs_utils::application;
use heroku_nodejs_utils::package_json::PackageJson;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::{layer_name, process_type};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
use libherokubuildpack::error::on_error as on_buildpack_error;
use libherokubuildpack::log::{log_header, log_info, log_warning};

pub(crate) struct NpmInstallBuildpack;

impl Buildpack for NpmInstallBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NpmInstallBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        context
            .app_dir
            .join("package.json")
            .exists()
            .then(|| {
                DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .provides("node_modules")
                            .requires("npm")
                            .requires("node_modules")
                            .requires("node")
                            .build(),
                    )
                    .build()
            })
            .unwrap_or_else(|| DetectResultBuilder::fail().build())
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        log_header("Heroku npm Install Buildpack");

        let env = Env::from_current();
        let app_dir = &context.app_dir;

        application::check_for_multiple_lockfiles(app_dir)
            .map_err(NpmInstallBuildpackError::Application)?;

        if let Some(warning) = application::warn_prebuilt_modules(app_dir) {
            log_warning(warning.header, warning.body);
        }

        let package_json = PackageJson::read(app_dir.join("package.json"))
            .map_err(NpmInstallBuildpackError::PackageJson)?;

        let npm_version = cmd::npm_version(&env).map_err(NpmInstallBuildpackError::NpmVersion)?;
        log_info(format!("npm version: {npm_version}"));

        let npm_cache_layer = context.handle_layer(layer_name!("npm_cache"), NpmCacheLayer {})?;

        cmd::npm_set_cache_config(&env, &npm_cache_layer.path)
            .map_err(NpmInstallBuildpackError::NpmSetCacheDir)?;

        cmd::npm_set_no_audit(&env).map_err(NpmInstallBuildpackError::NpmSetNoAudit)?;

        log_header("Installing dependencies");
        cmd::npm_install(&env).map_err(NpmInstallBuildpackError::NpmInstall)?;

        log_header("Running scripts");
        let build_scripts = package_json.build_scripts();
        if build_scripts.is_empty() {
            log_info("No build scripts found");
        } else {
            for build_script in build_scripts {
                log_info(format!("Running `{build_script}` script"));
                cmd::npm_run(&env, &build_script).map_err(NpmInstallBuildpackError::BuildScript)?;
            }
        }

        if package_json.has_start_script() {
            BuildResultBuilder::new()
                .launch(
                    LaunchBuilder::new()
                        .process(
                            ProcessBuilder::new(process_type!("web"), ["npm", "start"])
                                .default(true)
                                .build(),
                        )
                        .build(),
                )
                .build()
        } else {
            BuildResultBuilder::new().build()
        }
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        on_buildpack_error(log_user_errors, error);
    }
}

impl From<NpmInstallBuildpackError> for libcnb::Error<NpmInstallBuildpackError> {
    fn from(value: NpmInstallBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(NpmInstallBuildpack);
