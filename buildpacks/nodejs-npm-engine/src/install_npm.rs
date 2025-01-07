use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::{CommandWithName, NamedOutput};
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libherokubuildpack::download::{download_file, DownloadError};
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Stdout;
use std::path::Path;
use std::process::Command;

use heroku_nodejs_utils::inv::Release;
use heroku_nodejs_utils::vrs::Version;

use crate::errors::NpmEngineBuildpackError;
use crate::NpmEngineBuildpack;

pub(crate) fn install_npm(
    context: &BuildContext<NpmEngineBuildpack>,
    npm_release: &Release,
    node_version: &Version,
    mut logger: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, libcnb::Error<NpmEngineBuildpackError>> {
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
            logger = logger.sub_bullet("Using cached npm");
        }
        LayerState::Empty { ref cause } => {
            npm_engine_layer.write_metadata(new_metadata)?;

            if let EmptyLayerCause::RestoredLayerAction { cause } = cause {
                logger = logger.sub_bullet(format!(
                    "Invalidating cached npm ({} changed)",
                    cause.join(", ")
                ));
            }

            let downloaded_package_path = npm_engine_layer.path().join("npm.tgz");
            let npm_cli_script = npm_engine_layer.path().join("package/bin/npm-cli.js");

            // this install process is generalized from the npm install script at:
            // https://www.npmjs.com/install.sh
            logger = download_and_unpack_release(
                &npm_release.url,
                &downloaded_package_path,
                &npm_engine_layer.path(),
                logger,
            )?;
            logger = remove_existing_npm_installation(&npm_cli_script, logger)?;
            logger = install_npm_package(&npm_cli_script, &downloaded_package_path, logger)?;
        }
    }

    Ok(logger)
}

fn download_and_unpack_release(
    download_from: &String,
    download_to: &Path,
    unpack_into: &Path,
    logger: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, NpmEngineLayerError> {
    let timer = logger.start_timer(format!("Downloading {}", style::value(download_from)));
    download_file(download_from, download_to)
        .map_err(NpmEngineLayerError::Download)
        .and_then(|()| File::open(download_to).map_err(NpmEngineLayerError::OpenTarball))
        .and_then(|mut npm_tgz_file| {
            decompress_tarball(&mut npm_tgz_file, unpack_into)
                .map_err(NpmEngineLayerError::DecompressTarball)
        })?;
    Ok(timer.done())
}

fn remove_existing_npm_installation(
    npm_cli_script: &Path,
    mut logger: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, NpmEngineLayerError> {
    logger = logger.sub_bullet("Removing npm bundled with Node.js");
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
        .map(|_| logger)
}

fn install_npm_package(
    npm_cli_script: &Path,
    package: &Path,
    mut logger: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, NpmEngineLayerError> {
    logger = logger.sub_bullet("Installing requested npm");
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
        .map(|_| logger)
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
