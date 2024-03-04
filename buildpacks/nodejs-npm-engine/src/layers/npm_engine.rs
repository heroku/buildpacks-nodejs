use crate::errors::NpmEngineBuildpackError;
use crate::NpmEngineBuildpack;
use commons::output::fmt;
use commons::output::section_log::{log_step, log_step_timed, SectionLogger};
use fun_run::{CommandWithName, NamedOutput};
use heroku_nodejs_utils::inv::Release;
use heroku_nodejs_utils::vrs::Version;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::download::{download_file, DownloadError};
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::Path;
use std::process::Command;

/// A layer that downloads and sets up npm
pub(crate) struct NpmEngineLayer<'a> {
    pub(crate) npm_release: Release,
    pub(crate) node_version: Version,
    // this ensures we have a logging section already created
    pub(crate) _section_logger: &'a dyn SectionLogger,
}

const LAYER_VERSION: &str = "1";

impl<'a> Layer for NpmEngineLayer<'a> {
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
        &mut self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NpmEngineBuildpackError> {
        let downloaded_package_path = layer_path.join("npm.tgz");
        let npm_cli_script = layer_path.join("package/bin/npm-cli.js");

        // this install process is generalized from the npm install script at:
        // https://www.npmjs.com/install.sh
        download_and_unpack_release(&self.npm_release.url, &downloaded_package_path, layer_path)?;
        remove_existing_npm_installation(&npm_cli_script)?;
        install_npm_package(&npm_cli_script, &downloaded_package_path)?;

        LayerResultBuilder::new(NpmEngineLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &mut self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        let old_meta = &layer_data.content_metadata.metadata;
        let new_meta = &NpmEngineLayerMetadata::current(self, context);
        if old_meta == new_meta {
            log_step("Using cached npm");
            Ok(ExistingLayerStrategy::Keep)
        } else {
            log_step(format!(
                "Invalidating cached npm ({} changed)",
                changed_metadata_fields(old_meta, new_meta).join(", ")
            ));
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

fn download_and_unpack_release(
    download_from: &String,
    download_to: &Path,
    unpack_into: &Path,
) -> Result<(), NpmEngineLayerError> {
    log_step_timed(format!("Downloading {}", fmt::value(download_from)), || {
        download_file(download_from, download_to)
            .map_err(NpmEngineLayerError::Download)
            .and_then(|()| File::open(download_to).map_err(NpmEngineLayerError::OpenTarball))
            .and_then(|mut npm_tgz_file| {
                decompress_tarball(&mut npm_tgz_file, unpack_into)
                    .map_err(NpmEngineLayerError::DecompressTarball)
            })
    })
}

fn remove_existing_npm_installation(npm_cli_script: &Path) -> Result<(), NpmEngineLayerError> {
    log_step("Removing existing npm");
    Command::new("node")
        .args([
            &npm_cli_script.to_string_lossy(),
            "rm",
            "npm",
            "-gf",
            "--loglevel=silent",
        ])
        .named_output()
        .and_then(NamedOutput::nonzero_captured)
        .map_err(NpmEngineLayerError::RemoveExistingNpmInstall)
        .map(|_| ())
}

fn install_npm_package(npm_cli_script: &Path, package: &Path) -> Result<(), NpmEngineLayerError> {
    log_step("Installing requested npm");
    Command::new("node")
        .args([
            &npm_cli_script.to_string_lossy(),
            "install",
            "-gf",
            &package.to_string_lossy(),
        ])
        .named_output()
        .and_then(NamedOutput::nonzero_captured)
        .map_err(NpmEngineLayerError::InstallNpm)
        .map(|_| ())
}

fn changed_metadata_fields(
    old: &NpmEngineLayerMetadata,
    new: &NpmEngineLayerMetadata,
) -> Vec<String> {
    let mut changed = vec![];
    if old.npm_version != new.npm_version {
        changed.push("npm version".to_string());
    }
    if old.node_version != new.node_version {
        changed.push("node version".to_string());
    }
    if old.layer_version != new.layer_version {
        changed.push("layer version".to_string());
    }
    if old.os != new.os {
        changed.push("operating system".to_string());
    }
    if old.arch != new.arch {
        changed.push("compute architecture".to_string());
    }
    changed.sort();
    changed
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub(crate) struct NpmEngineLayerMetadata {
    layer_version: String,
    npm_version: String,
    node_version: String,
    arch: String,
    os: String,
}

impl NpmEngineLayerMetadata {
    fn current(layer: &NpmEngineLayer, context: &BuildContext<NpmEngineBuildpack>) -> Self {
        NpmEngineLayerMetadata {
            node_version: layer.node_version.to_string(),
            npm_version: layer.npm_release.version.to_string(),
            layer_version: String::from(LAYER_VERSION),
            arch: context.target.arch.clone(),
            os: context.target.os.clone(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum NpmEngineLayerError {
    Download(DownloadError),
    OpenTarball(std::io::Error),
    DecompressTarball(std::io::Error),
    RemoveExistingNpmInstall(fun_run::CmdError),
    InstallNpm(fun_run::CmdError),
}

impl From<NpmEngineLayerError> for NpmEngineBuildpackError {
    fn from(value: NpmEngineLayerError) -> Self {
        NpmEngineBuildpackError::NpmEngineLayer(value)
    }
}
