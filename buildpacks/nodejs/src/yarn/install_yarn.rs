use super::{YarnBuildpack, YarnBuildpackError};
use bullet_stream::global::print;
use heroku_nodejs_utils::http::{get, ResponseExt};
use heroku_nodejs_utils::npmjs_org::PackagePackument;
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

pub(crate) fn install_yarn(
    context: &BuildContext<YarnBuildpack>,
    yarn_package_packument: &PackagePackument,
) -> Result<LayerEnv, libcnb::Error<YarnBuildpackError>> {
    let new_metadata = CliLayerMetadata {
        yarn_version: yarn_package_packument.version.to_string(),
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
            print::sub_bullet(format!("Reusing yarn {}", yarn_package_packument.version));
        }
        LayerState::Empty { .. } => {
            dist_layer.write_metadata(new_metadata)?;

            let yarn_tgz = NamedTempFile::new().map_err(CliLayerError::TempFile)?;

            get(&yarn_package_packument.dist.tarball)
                .call_sync()
                .and_then(|res| res.download_to_file_sync(yarn_tgz.path()))
                .map_err(CliLayerError::Download)?;

            print::sub_bullet(format!(
                "Extracting yarn {}",
                yarn_package_packument.version
            ));
            decompress_tarball(&mut yarn_tgz.into_file(), dist_layer.path())
                .map_err(|e| CliLayerError::Untar(dist_layer.path(), e))?;

            print::sub_bullet(format!(
                "Installing yarn {}",
                yarn_package_packument.version
            ));

            let dist_name = if dist_layer.path().join("package").exists() {
                "package".to_string()
            } else {
                format!("yarn-v{}", yarn_package_packument.version)
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
    Download(heroku_nodejs_utils::http::Error),
    Untar(PathBuf, std::io::Error),
    Installation(std::io::Error),
    Permissions(std::io::Error),
}

impl From<CliLayerError> for libcnb::Error<YarnBuildpackError> {
    fn from(value: CliLayerError) -> Self {
        libcnb::Error::BuildpackError(YarnBuildpackError::CliLayer(value))
    }
}
