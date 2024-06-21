use crate::{YarnBuildpack, YarnBuildpackError};
use heroku_nodejs_utils::inv::Release;
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
#[allow(deprecated)]
use libcnb::layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder};
use libcnb::Buildpack;
use libherokubuildpack::download::{download_file, DownloadError};
use libherokubuildpack::fs::move_directory_contents;
use libherokubuildpack::log::log_info;
use libherokubuildpack::tar::decompress_tarball;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::NamedTempFile;
use thiserror::Error;

/// A layer that downloads and installs the yarn cli
pub(crate) struct CliLayer {
    pub(crate) release: Release,
}

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

const LAYER_VERSION: &str = "1";

#[allow(deprecated)]
impl Layer for CliLayer {
    type Buildpack = YarnBuildpack;
    type Metadata = CliLayerMetadata;

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
    ) -> Result<LayerResult<Self::Metadata>, YarnBuildpackError> {
        let yarn_tgz = NamedTempFile::new().map_err(CliLayerError::TempFile)?;

        log_info(format!("Downloading yarn {}", self.release.version));
        download_file(&self.release.url, yarn_tgz.path()).map_err(CliLayerError::Download)?;

        log_info(format!("Extracting yarn {}", self.release.version));
        decompress_tarball(&mut yarn_tgz.into_file(), layer_path).map_err(CliLayerError::Untar)?;

        log_info(format!("Installing yarn {}", self.release.version));

        let dist_name = if layer_path.join("package").exists() {
            "package".to_string()
        } else {
            format!("yarn-v{}", self.release.version)
        };

        move_directory_contents(layer_path.join(dist_name), layer_path)
            .map_err(CliLayerError::Installation)?;

        fs::set_permissions(
            layer_path.join("bin").join("yarn"),
            fs::Permissions::from_mode(0o755),
        )
        .map_err(CliLayerError::Permissions)?;

        LayerResultBuilder::new(CliLayerMetadata::current(self, context)).build()
    }

    fn existing_layer_strategy(
        &mut self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        if layer_data.content_metadata.metadata == CliLayerMetadata::current(self, context) {
            log_info(format!("Reusing yarn {}", self.release.version));
            Ok(ExistingLayerStrategy::Keep)
        } else {
            Ok(ExistingLayerStrategy::Recreate)
        }
    }
}

impl CliLayerMetadata {
    fn current(layer: &CliLayer, context: &BuildContext<YarnBuildpack>) -> Self {
        CliLayerMetadata {
            yarn_version: layer.release.version.to_string(),
            layer_version: String::from(LAYER_VERSION),
            arch: context.target.arch.clone(),
            os: context.target.os.clone(),
        }
    }
}
