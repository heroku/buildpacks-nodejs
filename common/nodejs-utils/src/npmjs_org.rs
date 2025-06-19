use crate::http::{get, ResponseExt};
use crate::vrs::{Requirement, Version};
use bullet_stream::global::print;
use http::{HeaderMap, HeaderValue, StatusCode};
use libcnb::build::BuildContext;
use libcnb::layer::{
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::Buildpack;
use libcnb_data::layer::{LayerName, LayerNameError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

const NPMJS_ORG_HOST: &str = "https://registry.npmjs.org";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackumentMetadata {
    etag: Option<String>,
    last_modified: Option<String>,
}

pub fn packument_layer<B, E>(
    context: &BuildContext<B>,
    package_name: impl AsRef<str>,
    on_error: impl Fn(PackumentLayerError) -> E,
) -> Result<Packument, libcnb::Error<B::Error>>
where
    B: Buildpack + Sized,
    E: Into<libcnb::Error<B::Error>>,
    libcnb::Error<<B as Buildpack>::Error>: From<E>,
{
    let packument_filename = "contents.json";

    let package_name = package_name.as_ref();

    // Handle package names with or without an org-scope as layer names. E.g.;
    // npm → npm_packment
    // @yarnpkg/cli-dist → yarnpkg_cli-dist_packument
    let layer_name = format!(
        "{}_packument",
        package_name.replace('/', "_").replace('@', "")
    )
    .parse::<LayerName>()
    .map_err(|e| on_error(PackumentLayerError::InvalidLayerName(e)))?;

    let packument_layer = context.cached_layer(
        layer_name,
        CachedLayerDefinition {
            build: true,
            launch: false,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|packument_metadata: &PackumentMetadata, layer_dir| {
                // make sure we can deserialize the packument file stored in the layer
                if parse_packument(&layer_dir.join(packument_filename)).is_ok() {
                    (RestoredLayerAction::KeepLayer, packument_metadata.clone())
                } else {
                    (
                        RestoredLayerAction::DeleteLayer,
                        PackumentMetadata {
                            etag: None,
                            last_modified: None,
                        },
                    )
                }
            },
        },
    )?;

    let packument_metadata = match packument_layer.state {
        LayerState::Restored {
            cause: ref packument_metadata,
        } => Some(packument_metadata),
        LayerState::Empty { .. } => None,
    };

    let mut headers = HeaderMap::new();
    if let Some(packument_metadata) = &packument_metadata {
        if let Some(etag) = &packument_metadata.etag {
            if let Ok(etag) = HeaderValue::from_str(etag) {
                headers.insert("If-None-Match", etag);
            }
        }
        if let Some(last_modified) = &packument_metadata.last_modified {
            if let Ok(last_modified) = HeaderValue::from_str(last_modified) {
                headers.insert("If-Modified-Since", last_modified);
            }
        }
    }

    let packument_response = get(format!("{NPMJS_ORG_HOST}/{package_name}"))
        .headers(headers)
        .call_sync()
        .map_err(|e| on_error(PackumentLayerError::FetchPackument(e)))?;

    let packument_file = packument_layer.path().join(packument_filename);

    // only update the metadata if we have a 200 response
    if packument_response.status() == StatusCode::OK {
        let etag = packument_response
            .headers()
            .get("ETag")
            .and_then(|value| value.to_str().map(ToString::to_string).ok());

        let last_modified = packument_response
            .headers()
            .get("Last-Modified")
            .and_then(|value| value.to_str().map(ToString::to_string).ok());

        packument_response
            .download_to_file_sync(&packument_file)
            .map_err(|e| on_error(PackumentLayerError::FetchPackument(e)))?;

        packument_layer.write_metadata(PackumentMetadata {
            etag,
            last_modified,
        })?;
    } else if packument_response.status() == StatusCode::NOT_MODIFIED {
        print::sub_bullet(format!("Using cached packument for {package_name}"));
    }

    parse_packument(&packument_file)
        .map_err(on_error)
        .map_err(Into::into)
}

fn parse_packument(packument_path: &Path) -> Result<Packument, PackumentLayerError> {
    fs::read_to_string(packument_path)
        .map_err(PackumentLayerError::ReadPackument)
        .and_then(|packument_contents| {
            serde_json::from_str::<Packument>(&packument_contents)
                .map_err(PackumentLayerError::ParsePackument)
        })
}

#[derive(Debug, Error)]
pub enum PackumentLayerError {
    #[error("Packument layer name error:\n{0}")]
    InvalidLayerName(LayerNameError),
    #[error("Packument request error:\n{0}")]
    FetchPackument(crate::http::Error),
    #[error("Packument read error:\n{0}")]
    ReadPackument(std::io::Error),
    #[error("Packument parse error:\n{0}")]
    ParsePackument(serde_json::Error),
}

#[derive(Deserialize, Clone)]
pub struct Packument {
    pub versions: HashMap<Version, PackagePackument>,
}

#[derive(Deserialize, Clone)]
pub struct PackagePackument {
    pub version: Version,
    pub dist: PackagePackumentDist,
}

#[derive(Deserialize, Clone)]
pub struct PackagePackumentDist {
    pub tarball: String,
}

#[must_use]
pub fn resolve_package_packument(
    packument: &Packument,
    requirement: &Requirement,
) -> Option<PackagePackument> {
    let mut package_packuments = packument.versions.values().cloned().collect::<Vec<_>>();

    // reverse sort, so latest is at the top
    package_packuments.sort_by(|a, b| b.version.cmp(&a.version));

    package_packuments
        .into_iter()
        .find(|package_packument| requirement.satisfies(&package_packument.version))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_packument_metadata() {
        let metadata = PackumentMetadata {
            etag: Some(String::from("\"fd20872fa73c2b790cc0eb7ff1bb42da\"")),
            last_modified: Some(String::from("Fri, 02 May 2025 13:50:44 GMT")),
        };

        let actual = toml::to_string(&metadata).unwrap();
        let expected = r#"
etag = '"fd20872fa73c2b790cc0eb7ff1bb42da"'
last_modified = "Fri, 02 May 2025 13:50:44 GMT"
"#
        .trim();
        assert_eq!(expected, actual.trim());
    }
}
