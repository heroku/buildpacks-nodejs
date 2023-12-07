mod errors;

use crate::errors::PnpmEngineBuildpackError;
use heroku_nodejs_utils::package_json::PackageJson;
use libcnb::build::{BuildContext, BuildResult};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use serde_json as _;
#[cfg(test)]
use test_support as _;

const BUILDPACK_NAME: &str = "Heroku Node.js pnpm Engine Buildpack";

struct PnpmEngineBuildpack;

impl Buildpack for PnpmEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = PnpmEngineBuildpackError;

    // scenarios with pnpm-lock.yaml
    // - package.json does not exist: fail
    // - package.json is not valid: error
    // - package.json does not have a packageManager key: error?
    // - package.json packageManager is not pnpm: error?
    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // This buildpack does not install pnpm yet. Currently, pnpm
        // installation happens only via heroku/nodejs-corepack. This
        // buildpack will error with guidance for apps with a `pnpm-lock.yaml`
        // that are missing the required corepack pnpm configuration.
        if context.app_dir.join("pnpm-lock.yaml").exists() {
            let package_json_path = context.app_dir.join("package.json");
            if package_json_path.exists()
                && PackageJson::read(package_json_path)
                    .map_err(PnpmEngineBuildpackError::PackageJson)?
                    .package_manager
                    .map_or(false, |pkg_mgr| pkg_mgr.name != "pnpm")
            {
                // Throw an error prior to attempting to build,
                Err(PnpmEngineBuildpackError::CorepackRequired)?
            }
        }
        DetectResultBuilder::fail().build()
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        // detect should error or fail. In the unexpected scenario that the
        // build is executed, we still want to suggest corepack instead.
        Err(PnpmEngineBuildpackError::CorepackRequired)?
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        errors::on_error(error);
    }
}

impl From<PnpmEngineBuildpackError> for libcnb::Error<PnpmEngineBuildpackError> {
    fn from(value: PnpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(PnpmEngineBuildpack);
