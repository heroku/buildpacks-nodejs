use std::fs::File;
use std::path::Path;
use std::process::Command;

use commons::output::fmt;
use commons::output::interface::SectionLogger;
use commons::output::section_log::{log_step, log_step_timed};
use fun_run::{CommandWithName, NamedOutput};
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libherokubuildpack::download::{download_file, DownloadError};
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};

use heroku_nodejs_utils::inv::Release;
use heroku_nodejs_utils::vrs::Version;

use crate::errors::NpmEngineBuildpackError;
use crate::NpmEngineBuildpack;

pub(crate) fn install_npm(
    context: &BuildContext<NpmEngineBuildpack>,
    npm_release: &Release,
    node_version: &Version,
    _logger: &dyn SectionLogger,
) -> Result<(), libcnb::Error<NpmEngineBuildpackError>> {
    let new_metadata = NpmEngineLayerMetadata {
        node_version: node_version.to_string(),
        npm_version: npm_release.version.to_string(),
        layer_version: LAYER_VERSION.to_string(),
        arch: context.target.arch.clone(),
        os: context.target.os.clone(),
    };

    let npm_engine_layer = context.cached_layer(
        layer_name!("npm_engine"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &NpmEngineLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    Ok((RestoredLayerAction::KeepLayer, vec![]))
                } else {
                    Ok((
                        RestoredLayerAction::DeleteLayer,
                        changed_metadata_fields(old_metadata, &new_metadata),
                    ))
                }
            },
        },
    )?;

    match npm_engine_layer.state {
        LayerState::Restored { .. } => {
            log_step("Using cached npm");
        }
        LayerState::Empty { ref cause } => {
            npm_engine_layer.write_metadata(new_metadata)?;

            if let EmptyLayerCause::RestoredLayerAction { cause } = cause {
                log_step(format!(
                    "Invalidating cached npm ({} changed)",
                    cause.join(", ")
                ));
            }

            let downloaded_package_path = npm_engine_layer.path().join("npm.tgz");
            let npm_cli_script = npm_engine_layer.path().join("package/bin/npm-cli.js");

            // this install process is generalized from the npm install script at:
            // https://www.npmjs.com/install.sh
            download_and_unpack_release(
                &npm_release.url,
                &downloaded_package_path,
                &npm_engine_layer.path(),
            )?;
            remove_existing_npm_installation(&npm_cli_script)?;
            install_npm_package(&npm_cli_script, &downloaded_package_path)?;
        }
    }

    Ok(())
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
    log_step("Removing npm bundled with Node.js");
    Command::new("node")
        .args([
            &npm_cli_script.to_string_lossy(),
            "rm",
            "npm",
            "-gf",
            "--loglevel=silent",
        ])
        .named_output()
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

const LAYER_VERSION: &str = "1";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NpmEngineLayerMetadata {
    layer_version: String,
    npm_version: String,
    node_version: String,
    arch: String,
    os: String,
}

#[derive(Debug)]
pub(crate) enum NpmEngineLayerError {
    Download(DownloadError),
    OpenTarball(std::io::Error),
    DecompressTarball(std::io::Error),
    RemoveExistingNpmInstall(fun_run::CmdError),
    InstallNpm(fun_run::CmdError),
}

impl From<NpmEngineLayerError> for libcnb::Error<NpmEngineBuildpackError> {
    fn from(value: NpmEngineLayerError) -> Self {
        libcnb::Error::BuildpackError(NpmEngineBuildpackError::NpmEngineLayer(value))
    }
}
