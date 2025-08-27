use super::main::NpmEngineBuildpackError;
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use fun_run::{CommandWithName, NamedOutput};
use heroku_nodejs_utils::http::{get, ResponseExt};
use heroku_nodejs_utils::npmjs_org::PackagePackument;
use heroku_nodejs_utils::vrs::Version;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn install_npm(
    context: &BuildpackBuildContext,
    npm_packument: &PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    let npm_version = &npm_packument.version;

    let new_metadata = NpmEngineLayerMetadata {
        node_version: node_version.to_string(),
        npm_version: npm_version.to_string(),
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
            print::sub_bullet("Using cached npm");
        }
        LayerState::Empty { ref cause } => {
            npm_engine_layer.write_metadata(new_metadata)?;

            if let EmptyLayerCause::RestoredLayerAction { cause } = cause {
                print::sub_bullet(format!(
                    "Invalidating cached npm ({} changed)",
                    cause.join(", ")
                ));
            }

            let downloaded_package_path = npm_engine_layer.path().join("npm.tgz");
            let npm_cli_script = npm_engine_layer.path().join("package/bin/npm-cli.js");

            // this install process is generalized from the npm install script at:
            // https://www.npmjs.com/install.sh
            download_and_unpack_release(
                &npm_packument.dist.tarball,
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
) -> Result<(), NpmInstallError> {
    get(download_from)
        .call_sync()
        .and_then(|res| res.download_to_file_sync(download_to))
        .map_err(NpmInstallError::Download)
        .and_then(|()| {
            File::open(download_to)
                .map_err(|e| NpmInstallError::OpenTarball(download_to.to_path_buf(), e))
        })
        .and_then(|mut npm_tgz_file| {
            decompress_tarball(&mut npm_tgz_file, unpack_into)
                .map_err(|e| NpmInstallError::DecompressTarball(download_to.to_path_buf(), e))
        })?;
    Ok(())
}

fn remove_existing_npm_installation(npm_cli_script: &Path) -> Result<(), NpmInstallError> {
    print::sub_bullet("Removing npm bundled with Node.js");
    Command::new("node")
        .args([
            &npm_cli_script.to_string_lossy(),
            "rm",
            "npm",
            "-gf",
            "--loglevel=silent",
        ])
        .named_output()
        .map_err(NpmInstallError::RemoveExistingNpmInstall)
        .map(|_| ())
}

fn install_npm_package(npm_cli_script: &Path, package: &Path) -> Result<(), NpmInstallError> {
    print::sub_bullet("Installing requested npm");
    Command::new("node")
        .args([
            &npm_cli_script.to_string_lossy(),
            "install",
            "-gf",
            &package.to_string_lossy(),
        ])
        .named_output()
        .and_then(NamedOutput::nonzero_captured)
        .map_err(NpmInstallError::InstallNpm)
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
pub(crate) enum NpmInstallError {
    Download(heroku_nodejs_utils::http::Error),
    OpenTarball(PathBuf, std::io::Error),
    DecompressTarball(PathBuf, std::io::Error),
    RemoveExistingNpmInstall(fun_run::CmdError),
    InstallNpm(fun_run::CmdError),
}

impl From<NpmInstallError> for BuildpackError {
    fn from(value: NpmInstallError) -> Self {
        NpmEngineBuildpackError::NpmInstall(value).into()
    }
}
