use crate::cmd;
use crate::layers::npm_engine::NpmEngineLayerError;
use heroku_nodejs_utils::package_json::PackageJsonError;
use heroku_nodejs_utils::vrs::Requirement;
use libcnb::Error;

#[derive(Debug)]
pub(crate) enum NpmEngineBuildpackError {
    PackageJson(PackageJsonError),
    MissingNpmEngineRequirement,
    InventoryParse(toml::de::Error),
    NpmVersionResolve(Requirement),
    NpmSetupLayer(NpmEngineLayerError),
    NodeVersionCommand(cmd::Error),
    NpmVersionCommand(cmd::Error),
}

pub(crate) fn on_error(_error: Error<NpmEngineBuildpackError>) {
    todo!()
}
