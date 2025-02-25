use bullet_stream::state::SubBullet;
use bullet_stream::Print;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::LayerEnv;
use libherokubuildpack::download::{download_file, DownloadError};
use libherokubuildpack::fs::move_directory_contents;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Stdout;
use std::os::unix::fs::PermissionsExt;
use tempfile::NamedTempFile;
use thiserror::Error;

use heroku_nodejs_utils::inv::Release;

use crate::{YarnBuildpack, YarnBuildpackError};

pub(crate) fn install_yarn(
    context: &BuildContext<YarnBuildpack>,
    release: &Release,
    mut log: Print<SubBullet<Stdout>>,
) -> Result<(LayerEnv, Print<SubBullet<Stdout>>), libcnb::Error<YarnBuildpackError>> {
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
            log = log.sub_bullet(format!("Reusing yarn {}", release.version));
        }
        LayerState::Empty { .. } => {
            dist_layer.write_metadata(new_metadata)?;

            let yarn_tgz = NamedTempFile::new().map_err(CliLayerError::TempFile)?;

            let timer = log.start_timer(format!("Downloading yarn {}", release.version));
            download_file(&release.url, yarn_tgz.path()).map_err(CliLayerError::Download)?;
            log = timer.done();

            log = log.sub_bullet(format!("Extracting yarn {}", release.version));
            decompress_tarball(&mut yarn_tgz.into_file(), dist_layer.path())
                .map_err(CliLayerError::Untar)?;

            log = log.sub_bullet(format!("Installing yarn {}", release.version));

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

    dist_layer.read_env().map(|env| (env, log))
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

#[derive(Error, Debug)]
pub(crate) enum CliLayerError {
    #[error("Couldn't create tempfile for yarn CLI: {0}")]
    TempFile(std::io::Error),
    #[error("Couldn't download yarn CLI: {0}")]
    Download(DownloadError),
    #[error("Couldn't decompress yarn CLI: {0}")]
    Untar(std::io::Error),
    #[error("Couldn't move yarn CLI to the target location: {0}")]
    Installation(std::io::Error),
    #[error("Couldn't set CLI permissions: {0}")]
    Permissions(std::io::Error),
}

impl From<CliLayerError> for libcnb::Error<YarnBuildpackError> {
    fn from(value: CliLayerError) -> Self {
        libcnb::Error::BuildpackError(YarnBuildpackError::CliLayer(value))
    }
}
