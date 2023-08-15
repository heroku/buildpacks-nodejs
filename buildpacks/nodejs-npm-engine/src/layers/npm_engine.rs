use crate::errors::NpmEngineBuildpackError;
use crate::NpmEngineBuildpack;
use heroku_nodejs_utils::inv::Release;
use heroku_nodejs_utils::vrs::Version;
use libcnb::build::BuildContext;
use libcnb::data::buildpack::StackId;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::download::{download_file, DownloadError};
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::process::{Command, Stdio};

/// A layer that downloads and sets up npm
pub(crate) struct NpmEngineLayer {
    pub npm_release: Release,
    pub node_version: Version,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct NpmEngineLayerMetadata {
    layer_version: String,
    npm_version: String,
    node_version: String,
    stack_id: StackId,
}

const LAYER_VERSION: &str = "1";

impl Layer for NpmEngineLayer {
    type Buildpack = NpmEngineBuildpack;
    type Metadata = NpmEngineLayerMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NpmEngineBuildpackError> {
        // this install process is generalized from the npm install script at:
        // https://www.npmjs.com/install.sh
        let npm_tgz = layer_path.join("npm.tgz");

        log_info("Downloading and extracting npm...");
        download_file(&self.npm_release.url, &npm_tgz).map_err(NpmEngineLayerError::Download)?;

        let mut npm_tgz_file = File::open(&npm_tgz).map_err(NpmEngineLayerError::OpenTarball)?;
        decompress_tarball(&mut npm_tgz_file, layer_path)
            .map_err(NpmEngineLayerError::DecompressTarball)?;

        let npm_cli_script = layer_path.join("package/bin/npm-cli.js");

        Command::new("node")
            .args([&npm_cli_script.to_string_lossy(), "rm", "npm", "-gf"])
            .output()
            .map_err(NpmEngineLayerError::RemoveExistingNpmInstall)?;

        Command::new("node")
            .args([
                &npm_cli_script.to_string_lossy(),
                "install",
                "-gf",
                &npm_tgz.to_string_lossy(),
            ])
            .stdout(Stdio::piped())
            .output()
            .map_err(NpmEngineLayerError::InstallNpm)?;

        LayerResultBuilder::new(NpmEngineLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata == NpmEngineLayerMetadata::current(self, context) {
            log_info("Reusing cached npm...");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl NpmEngineLayerMetadata {
    fn current(layer: &NpmEngineLayer, context: &BuildContext<NpmEngineBuildpack>) -> Self {
        NpmEngineLayerMetadata {
            node_version: layer.node_version.to_string(),
            npm_version: layer.npm_release.version.to_string(),
            stack_id: context.stack_id.clone(),
            layer_version: String::from(LAYER_VERSION),
        }
    }
}

#[derive(Debug)]
pub(crate) enum NpmEngineLayerError {
    Download(DownloadError),
    OpenTarball(std::io::Error),
    DecompressTarball(std::io::Error),
    RemoveExistingNpmInstall(std::io::Error),
    InstallNpm(std::io::Error),
}

impl From<NpmEngineLayerError> for NpmEngineBuildpackError {
    fn from(value: NpmEngineLayerError) -> Self {
        NpmEngineBuildpackError::NpmSetupLayer(value)
    }
}
