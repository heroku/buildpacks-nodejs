use bullet_stream::global::print;
use bullet_stream::style;
use heroku_nodejs_utils::download_file::{download_file_sync, DownloadError};
use heroku_nodejs_utils::inv::Release;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::LayerEnv;
use libherokubuildpack::fs::move_directory_contents;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::NamedTempFile;

use crate::{YarnBuildpack, YarnBuildpackError};

pub(crate) fn install_yarn(
    context: &BuildContext<YarnBuildpack>,
    release: &Release,
) -> Result<LayerEnv, libcnb::Error<YarnBuildpackError>> {
    let new_metadata = CliLayerMetadata {
        yarn_version: release.version.to_string(),
        layer_version: LAYER_VERSION.to_string(),
        arch: context.target.arch.clone(),
        os: context.target.os.clone(),
    };

    let dist_layer = context.cached_layer(
        layer_name!("dist"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &CliLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match dist_layer.state {
        LayerState::Restored { .. } => {
            print::sub_bullet(format!("Reusing yarn {}", release.version));
        }
        LayerState::Empty { .. } => {
            dist_layer.write_metadata(new_metadata)?;

            let yarn_tgz = NamedTempFile::new().map_err(CliLayerError::TempFile)?;

            download_file_sync()
                .downloading_message(format!(
                    "Downloading Yarn from {}",
                    style::url(&release.url)
                ))
                .from_url(&release.url)
                .to_file(yarn_tgz.path())
                .call()
                .map_err(CliLayerError::Download)?;

            print::sub_bullet(format!("Extracting yarn {}", release.version));
            decompress_tarball(&mut yarn_tgz.into_file(), dist_layer.path())
                .map_err(|e| CliLayerError::Untar(dist_layer.path(), e))?;

            print::sub_bullet(format!("Installing yarn {}", release.version));

            let dist_name = if dist_layer.path().join("package").exists() {
                "package".to_string()
            } else {
                format!("yarn-v{}", release.version)
            };

            move_directory_contents(dist_layer.path().join(dist_name), dist_layer.path())
                .map_err(CliLayerError::Installation)?;

            fs::set_permissions(
                dist_layer.path().join("bin").join("yarn"),
                fs::Permissions::from_mode(0o755),
            )
            .map_err(CliLayerError::Permissions)?;
        }
    }

    dist_layer.read_env()
}

const LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct CliLayerMetadata {
    layer_version: String,
    yarn_version: String,
    arch: String,
    os: String,
}

#[derive(Debug)]
pub(crate) enum CliLayerError {
    TempFile(std::io::Error),
    Download(DownloadError),
    Untar(PathBuf, std::io::Error),
    Installation(std::io::Error),
    Permissions(std::io::Error),
}

impl From<CliLayerError> for libcnb::Error<YarnBuildpackError> {
    fn from(value: CliLayerError) -> Self {
        libcnb::Error::BuildpackError(YarnBuildpackError::CliLayer(value))
    }
}
