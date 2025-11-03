use crate::utils::download::{Downloader, DownloaderError, Extractor, GzipOptions, download_sync};
use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::http::{ResponseExt, get};
use crate::utils::vrs::{Requirement, Version};
use crate::{BuildpackBuildContext, BuildpackError, utils};
use bullet_stream::global::print;
use bullet_stream::style;
use http::{HeaderMap, HeaderValue, StatusCode};
use indoc::formatdoc;
use libcnb::Env;
use libcnb::data::layer::LayerName;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::Scope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const NPMJS_ORG_HOST: &str = "https://registry.npmjs.org";

const NPM_STATUS_URL: &str = "https://status.npmjs.org/";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PackumentMetadata {
    etag: Option<String>,
    last_modified: Option<String>,
}

pub(crate) fn packument_layer(
    layer_name: LayerName,
    context: &BuildpackBuildContext,
    package_name: impl AsRef<str>,
) -> Result<Packument, PackumentLayerError> {
    let packument_filename = "contents.json";

    let package_name = package_name.as_ref();

    let parse_packument = |packument_path: &Path| {
        fs::read_to_string(packument_path)
            .map_err(|e| PackumentLayerError::ReadPackument(package_name.to_string(), e))
            .and_then(|packument_contents| {
                serde_json::from_str::<Packument>(&packument_contents)
                    .map_err(|e| PackumentLayerError::ParsePackument(package_name.to_string(), e))
            })
    };

    let packument_layer = context
        .cached_layer(
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
        )
        .map_err(|e| PackumentLayerError::Layer(Box::new(e)))?;

    let packument_metadata = match packument_layer.state {
        LayerState::Restored {
            cause: ref packument_metadata,
        } => Some(packument_metadata),
        LayerState::Empty { .. } => None,
    };

    let mut headers = HeaderMap::new();

    if let Some(packument_metadata) = &packument_metadata {
        if let Some(etag) = &packument_metadata.etag
            && let Ok(etag) = HeaderValue::from_str(etag)
        {
            headers.insert("If-None-Match", etag);
        }
        if let Some(last_modified) = &packument_metadata.last_modified
            && let Ok(last_modified) = HeaderValue::from_str(last_modified)
        {
            headers.insert("If-Modified-Since", last_modified);
        }
    }

    let packument_response = get(format!("{NPMJS_ORG_HOST}/{package_name}"))
        .headers(headers)
        .call_sync()
        .map_err(|e| PackumentLayerError::FetchPackument(package_name.to_string(), e))?;

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
            .map_err(|e| PackumentLayerError::FetchPackument(package_name.to_string(), e))?;

        packument_layer
            .write_metadata(PackumentMetadata {
                etag,
                last_modified,
            })
            .map_err(|e| PackumentLayerError::Layer(Box::new(e)))?;
    } else if packument_response.status() == StatusCode::NOT_MODIFIED {
        print::sub_bullet(format!("Using cached packument for {package_name}"));
    } else {
        Err(PackumentLayerError::UnexpectedResponse(
            package_name.to_string(),
            packument_response.status(),
        ))?;
    }

    parse_packument(&packument_file)
}

#[derive(Debug)]
pub(crate) enum PackumentLayerError {
    FetchPackument(String, utils::http::Error),
    UnexpectedResponse(String, StatusCode),
    ReadPackument(String, std::io::Error),
    ParsePackument(String, serde_json::Error),
    Layer(Box<BuildpackError>),
}

impl From<PackumentLayerError> for BuildpackError {
    fn from(value: PackumentLayerError) -> Self {
        match value {
            PackumentLayerError::FetchPackument(package_name, e) => {
                create_packument_layer_fetch_packument_error(&package_name, &e).into()
            }
            PackumentLayerError::UnexpectedResponse(package_name, status_code) => {
                create_packument_layer_unexpected_response_error(&package_name, status_code).into()
            }
            PackumentLayerError::ReadPackument(package_name, error) => {
                create_packument_layer_internal_error(&package_name, &error.to_string()).into()
            }
            PackumentLayerError::ParsePackument(package_name, error) => {
                create_packument_layer_internal_error(&package_name, &error.to_string()).into()
            }
            PackumentLayerError::Layer(error) => *error,
        }
    }
}

fn create_packument_layer_fetch_packument_error(
    package_name: &str,
    error: &utils::http::Error,
) -> ErrorMessage {
    let package_name = style::value(package_name);
    let npm_status_url = style::url(NPM_STATUS_URL);
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header(format!("Failed to load available {package_name} versions"))
        .body(formatdoc! { "
            An unexpected error occurred while loading the available {package_name} versions. This error can \
            occur due to an unstable network connection or an issue with the npm registry.

            Suggestions:
            - Check the npm status page for any ongoing incidents ({npm_status_url})
        "})
        .debug_info(error.to_string())
        .create()
}

fn create_packument_layer_unexpected_response_error(
    package_name: &str,
    status_code: StatusCode,
) -> ErrorMessage {
    let status_code = style::value(status_code.as_str());
    let ok = style::value("200 (OK)");
    let not_modified = style::value("304 (Not Modified)");
    create_packument_layer_internal_error(
        package_name,
        &formatdoc! { "
        An unexpected response status was received while loading the available {package_name} versions. The
        status code returned was {status_code} but a {ok} or {not_modified} status code was expected.
    " },
    )
}

fn create_packument_layer_internal_error(package_name: &str, debug_info: &str) -> ErrorMessage {
    let package_name = style::value(package_name);
    let npm_status_url = style::url(NPM_STATUS_URL);
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
        .header(format!("Failed to load available {package_name} versions"))
        .body(formatdoc! { "
            An unexpected error occurred while loading the available {package_name} versions from the npm registry.
            See the debug info above for more details.

            Suggestions:
            - Check the npm status page for any ongoing incidents ({npm_status_url})
        "})
        .debug_info(debug_info)
        .create()
}

#[derive(Deserialize, Clone)]
pub(crate) struct Packument {
    pub(crate) name: String,
    pub(crate) versions: HashMap<Version, PackagePackument>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct PackagePackument {
    pub(crate) name: String,
    pub(crate) version: Version,
    pub(crate) dist: PackagePackumentDist,
    pub(crate) bin: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct PackagePackumentDist {
    pub(crate) tarball: String,
}

pub(crate) fn resolve_package_packument(
    packument: &Packument,
    requirement: &Requirement,
) -> Result<PackagePackument, ErrorMessage> {
    let mut package_packuments = packument.versions.values().cloned().collect::<Vec<_>>();

    // reverse sort, so latest is at the top
    package_packuments.sort_by(|a, b| b.version.cmp(&a.version));

    package_packuments
        .into_iter()
        .find(|package_packument| requirement.satisfies(&package_packument.version))
        .ok_or_else(|| create_resolve_package_packument_error(packument, requirement))
}

fn create_resolve_package_packument_error(
    packument: &Packument,
    requirement: &Requirement,
) -> ErrorMessage {
    let package_name = &packument.name;
    let npm_releases_url =
        format!("https://www.npmjs.com/package/{package_name}?activeTab=versions");
    let npm_show_command = format!("npm show '{package_name}@{requirement}' versions");

    let package_name = style::value(package_name);
    let requested_version = style::value(requirement.to_string());
    let npm_releases_url = style::url(npm_releases_url);
    let npm_show_command = style::value(npm_show_command);

    error_message()
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!("Error resolving requested {package_name} version {requested_version}"))
        .body(formatdoc! { "
            The requested {package_name} version could not be resolved to a specific version from the \
            npm registry.

            Suggestions:
            - Confirm if this is a valid {package_name} release at {npm_releases_url} or by running {npm_show_command}
        " })
        .create()
}

pub(crate) fn install_package_layer(
    layer_name: LayerName,
    context: &BuildpackBuildContext,
    env: &mut Env,
    package_packument: &PackagePackument,
    node_version: &Version,
) -> Result<(), InstallPackageLayerError> {
    let package_name = &package_packument.name;
    let package_version = &package_packument.version;

    let new_metadata = InstallPackageLayerMetadata {
        node_version: node_version.to_string(),
        package_name: package_name.clone(),
        package_version: package_version.to_string(),
        layer_version: INSTALL_PACKAGE_LAYER_VERSION.to_string(),
        arch: context.target.arch.clone(),
        os: context.target.os.clone(),
    };

    let install_package_layer = context
        .cached_layer(
            layer_name,
            CachedLayerDefinition {
                build: true,
                launch: true,
                invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
                restored_layer_action: &|old_metadata: &InstallPackageLayerMetadata, _| {
                    if old_metadata == &new_metadata {
                        Ok((RestoredLayerAction::KeepLayer, vec![]))
                    } else {
                        Ok((
                            RestoredLayerAction::DeleteLayer,
                            install_layer_changed_metadata_fields(old_metadata, &new_metadata),
                        ))
                    }
                },
            },
        )
        .map_err(|e| InstallPackageLayerError::Layer(Box::new(e)))?;

    match install_package_layer.state {
        LayerState::Restored { .. } => {
            print::sub_bullet(format!("Using cached version of {package_name}"));
        }
        LayerState::Empty { ref cause } => {
            if let EmptyLayerCause::RestoredLayerAction { cause } = cause {
                print::sub_bullet(format!(
                    "Invalidating cached {package_name} ({} changed)",
                    cause.join(", ")
                ));
            }

            download_sync(PackageDownloader {
                url: package_packument.dist.tarball.clone(),
                destination: install_package_layer.path(),
            })
            .map_err(|e| {
                InstallPackageLayerError::Download(Box::new(package_packument.clone()), e)
            })?;

            // Create symlinks for all binaries declared in packument's `bin` field
            let bins = package_packument.bin.as_ref().ok_or_else(|| {
                InstallPackageLayerError::MissingBins(Box::new(package_packument.clone()))
            })?;
            for (name, script) in bins {
                let bin_path = install_package_layer.path().join("bin").join(name);
                let script_path = install_package_layer.path().join(script);
                match std::fs::remove_file(&bin_path) {
                    Ok(()) => Ok(()),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(e) => Err(InstallPackageLayerError::WriteBin(
                        Box::new(package_packument.clone()),
                        e,
                    )),
                }?;
                std::os::unix::fs::symlink(script_path, bin_path).map_err(|e| {
                    InstallPackageLayerError::WriteBin(Box::new(package_packument.clone()), e)
                })?;
            }

            install_package_layer
                .write_metadata(new_metadata)
                .map_err(|e| InstallPackageLayerError::Layer(Box::new(e)))?;
        }
    }

    let layer_env = &install_package_layer
        .read_env()
        .map_err(|e| InstallPackageLayerError::Layer(Box::new(e)))?;

    env.clone_from(&layer_env.apply(Scope::Build, env));

    Ok(())
}

pub(crate) enum InstallPackageLayerError {
    Download(Box<PackagePackument>, DownloaderError),
    Layer(Box<libcnb::Error<ErrorMessage>>),
    MissingBins(Box<PackagePackument>),
    WriteBin(Box<PackagePackument>, std::io::Error),
}

impl From<InstallPackageLayerError> for BuildpackError {
    fn from(value: InstallPackageLayerError) -> Self {
        match value {
            InstallPackageLayerError::Download(package_packument, error) => {
                create_install_package_download_error(&package_packument, &error).into()
            }
            InstallPackageLayerError::MissingBins(package_packument) => {
                create_install_package_missing_bins_error(&package_packument).into()
            }
            InstallPackageLayerError::WriteBin(package_packument, error) => {
                create_install_package_write_error(&package_packument, &error).into()
            }
            InstallPackageLayerError::Layer(error) => *error,
        }
    }
}

fn create_install_package_download_error(
    package_packument: &PackagePackument,
    error: &DownloaderError,
) -> ErrorMessage {
    let npm_status_url = style::url(NPM_STATUS_URL);
    match error {
        DownloaderError::Request { source, .. } => {
            let package_name = style::value(&package_packument.name);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header(format!("Failed to download {package_name}"))
                .body(formatdoc! {"
                    An unexpected error occurred while downloading the {package_name} package. This error can \
                    occur due to an unstable network connection or an issue with the npm registry.

                    Suggestions:
                    - Check the npm status page for any ongoing incidents ({npm_status_url})
                " })
                .debug_info(source.to_string())
                .create()
        }

        DownloaderError::ChecksumMismatch { .. } => {
            unreachable!("The package installation layer does not perform checksums currently")
        }

        DownloaderError::Write {
            destination,
            source,
            ..
        } => {
            let package_name = style::value(&package_packument.name);
            let destination = file_value(destination);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header(format!("Failed to extract {package_name} package file"))
                .body(formatdoc! {"
                    An unexpected I/O occurred while extracting the contents of the downloaded {package_name} package file to {destination}.
                " })
                .debug_info(source.to_string())
                .create()
        }
    }
}

fn create_install_package_write_error(
    package_packument: &PackagePackument,
    error: &std::io::Error,
) -> ErrorMessage {
    let package_name = style::value(&package_packument.name);
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
        .header(format!(
            "Failed to install the downloaded {package_name} package binaries"
        ))
        .body(formatdoc! { "
            An unexpected error occurred while linking the {package_name} package scripts to \
            their declared binaries.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_install_package_missing_bins_error(package_packument: &PackagePackument) -> ErrorMessage {
    let package_name = style::value(&package_packument.name);
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!(
            "Failed to determine the downloaded {package_name} package binaries"
        ))
        .body(formatdoc! {"
            The downloaded {package_name} package does not declare any binaries to install. This is \
            unexpected and may indicate a bug in the buildpack.
        " })
        .create()
}

struct PackageDownloader {
    url: String,
    destination: PathBuf,
}

impl Downloader<'_> for PackageDownloader {
    fn source_url(&self) -> &str {
        &self.url
    }

    fn destination(&self) -> &Path {
        &self.destination
    }

    fn extractor(&self) -> Option<Extractor> {
        Some(Extractor::Gzip(GzipOptions {
            strip_components: 1,
            exclude: Box::new(|path| {
                path.components().take(1).next().is_some_and(|c| {
                    // test
                    c.as_os_str() == "docs" || c.as_os_str() == "man"
                })
            }),
            ..GzipOptions::default()
        }))
    }
}

fn install_layer_changed_metadata_fields(
    old: &InstallPackageLayerMetadata,
    new: &InstallPackageLayerMetadata,
) -> Vec<String> {
    let mut changed = vec![];
    if old.package_name != new.package_name {
        changed.push("package name".to_string());
    }
    if old.package_version != new.package_version {
        changed.push("version".to_string());
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
        changed.push("cpu architecture".to_string());
    }
    changed.sort();
    changed
}

const INSTALL_PACKAGE_LAYER_VERSION: &str = "1";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstallPackageLayerMetadata {
    layer_version: String,
    package_name: String,
    package_version: String,
    node_version: String,
    arch: String,
    os: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_reqwest_error};
    use std::str::FromStr;

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

    #[test]
    fn test_install_package_layer_metadata() {
        let metadata = InstallPackageLayerMetadata {
            layer_version: INSTALL_PACKAGE_LAYER_VERSION.to_string(),
            package_name: "test".to_string(),
            package_version: "1.0.0".to_string(),
            node_version: "18.16.0".to_string(),
            arch: "x64".to_string(),
            os: "linux".to_string(),
        };

        let actual = toml::to_string(&metadata).unwrap();
        let expected = r#"
layer_version = "1"
package_name = "test"
package_version = "1.0.0"
node_version = "18.16.0"
arch = "x64"
os = "linux"
"#
        .trim();
        assert_eq!(expected, actual.trim());
    }

    #[bon::builder(on(String, into), finish_fn = build)]
    #[allow(clippy::needless_pass_by_value)]
    fn package_packument(
        #[builder(start_fn)] //
        name: String,
        #[builder(default = "1.0.0")] //
        version: String,
        #[builder(default = HashMap::new())] //
        bin: HashMap<String, String>,
    ) -> PackagePackument {
        let tarball = format!("https://registry.npmjs.org/{name}/-/{name}-{version}.tgz");
        PackagePackument {
            name,
            version: Version::from_str(&version).unwrap(),
            dist: PackagePackumentDist { tarball },
            bin: Some(bin),
        }
    }

    #[test]
    fn test_install_package_download_request_error() {
        let package_packument = package_packument("test").build();
        assert_error_snapshot(&create_install_package_download_error(
            &package_packument,
            &DownloaderError::Request {
                url: package_packument.dist.tarball.clone(),
                source: crate::utils::http::Error::Request(
                    package_packument.dist.tarball.clone(),
                    create_reqwest_error(),
                ),
            },
        ));
    }

    #[test]
    fn test_install_package_download_write_error() {
        let package_packument = package_packument("test").build();
        assert_error_snapshot(&create_install_package_download_error(
            &package_packument,
            &DownloaderError::Write {
                url: package_packument.dist.tarball.clone(),
                source: std::io::Error::other("Out of disk space"),
                destination: format!("/layers/nodejs/{}", &package_packument.name).into(),
            },
        ));
    }
}
