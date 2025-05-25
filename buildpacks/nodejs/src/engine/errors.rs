use crate::engine::install_node::DistLayerError;
use crate::NodeJsEngineBuildpackError;
use bullet_stream::style;
use heroku_nodejs_utils::download_file::DownloadError;
use heroku_nodejs_utils::error_handling::error_message_builder::SetIssuesUrl;
use heroku_nodejs_utils::error_handling::ErrorType::{Internal, UserFacing};
use heroku_nodejs_utils::error_handling::{
    file_value, on_package_json_error, ErrorMessage, ErrorMessageBuilder, SuggestRetryBuild,
    SuggestSubmitIssue,
};
use indoc::formatdoc;

const BUILDPACK_NAME: &str = "Heroku Node.js Engine buildpack";

const ISSUES_URL: &str = "https://github.com/heroku/buildpacks-nodejs/issues";

// Wraps the error_message() builder to preset the issues_url field
fn error_message() -> ErrorMessageBuilder<SetIssuesUrl> {
    heroku_nodejs_utils::error_handling::error_message().issues_url(ISSUES_URL.to_string())
}

pub(crate) fn on_nodejs_engine_buildpack_error(error: NodeJsEngineBuildpackError) -> ErrorMessage {
    match error {
        NodeJsEngineBuildpackError::InventoryParse(e) => on_inventory_parse_error(&e),
        NodeJsEngineBuildpackError::PackageJson(e) => {
            on_package_json_error(BUILDPACK_NAME, ISSUES_URL, e)
        }
        NodeJsEngineBuildpackError::UnknownVersion(e) => on_unknown_version_error(e),
        NodeJsEngineBuildpackError::DistLayer(e) => on_dist_layer_error(e),
    }
}

fn on_inventory_parse_error(error: &toml::de::Error) -> ErrorMessage {
    let inventory_file = file_value("./inventory.toml");
    error_message()
        .error_type(Internal)
        .header("Couldn't parse Node.js inventory")
        .body(formatdoc! {"
            The {BUILDPACK_NAME} embeds the content of the {inventory_file} found in this buildpack's \
            repository into its binary to look up and resolve Node.js versions but the content isn't \
            valid TOML.
        "})
        .debug_info(error.to_string())
        .create()
}

fn on_unknown_version_error(version: String) -> ErrorMessage {
    let node_releases_url = style::url(format!(
        "https://github.com/nodejs/node/releases?q=v{version}&expanded=true"
    ));
    let inventory_url = style::url("https://github.com/heroku/buildpacks-nodejs/blob/main/buildpacks/nodejs-engine/inventory.toml");
    let version = style::value(version);
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!("Unknown Node.js version: {version}"))
        .body(formatdoc! {"
            The Node.js version provided could not be resolved to a known release in this buildpack's \
            inventory of Node.js releases.
            
            Suggestions:
            - Confirm if this is a valid Node.js release at {node_releases_url}
            - Check if this buildpack includes the requested Node.js version in its inventory file at {inventory_url}     
        "})
        .create()
}

#[allow(clippy::too_many_lines)]
fn on_dist_layer_error(error: DistLayerError) -> ErrorMessage {
    match error {
        DistLayerError::TempFile(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
            .header("Failed to create temporary download file")
            .body(formatdoc! { "
                The {BUILDPACK_NAME} downloads the target Node.js distribution to a temporary file \
                before installation but an unexpected error occurred.
            "})
            .debug_info(e.to_string())
            .create(),

        DistLayerError::Download {
            src_url,
            dst_path,
            source,
        } => {
            let nodejs_status_url = style::url("https://status.nodejs.org/");
            let src_url = style::url(src_url);
            let dst_path = file_value(dst_path);
            match source {
                DownloadError::Request(_, _) | DownloadError::ReadResponse(_, _) => error_message()
                    .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                    .header("Failed to download Node.js distribution")
                    .body(formatdoc! {"
                        A request to download the target Node.js distribution from {src_url} failed unexpectedly. This error can \
                        occur due to an unstable network connection or an issue with the upstream Node.js distribution \
                        server.
                        
                        Suggestions:
                        - Check the status of {nodejs_status_url} for any reported issues.
                        - Confirm the download url ({src_url}) works. 
                    "})
                    .debug_info(source.to_string())
                    .create(),

                DownloadError::OpenFile(_, _, _) | DownloadError::WriteFile(_, _, _) => error_message()
                    .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                    .header("Failed to write downloaded Node.js distribution")
                    .body(formatdoc! {"
                        An unexpected I/O error occurred while writing the Node.js distribution from {src_url} to \
                        disk at {dst_path}.
                    "})
                    .debug_info(source.to_string())
                    .create()
            }
        }

        DistLayerError::Untar {
            src_path,
            dst_path,
            source,
        } => {
            let src_path = file_value(src_path);
            let dst_path = file_value(dst_path);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header("Failed to unpack Node.js distribution")
                .body(formatdoc! {"
                    An unexpected I/O error occurred while trying to unpack from {src_path} to {dst_path}.
                "})
                .debug_info(source.to_string())
                .create()
        }

        DistLayerError::TarballPrefix(url) => {
            let url = style::url(url);
            error_message()
                .error_type(Internal)
                .header("Could not determine tarball extraction directory")
                .body(formatdoc! {"
                    An unexpected error occurred while trying to determine the name of the tarball \
                    extraction directory from {url}.
                "})
                .create()
        }

        DistLayerError::Installation {
            src_path,
            dst_path,
            source,
        } => {
            let src_path = file_value(src_path);
            let dst_path = file_value(dst_path);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header("Failed to copy Node.js distribution contents")
                .body(formatdoc! {"
                    An unexpected I/O error occurred while copying the contents of the Node.js \
                    distribution from {src_path} to the installation directory at {dst_path}. 
                "})
                .debug_info(source.to_string())
                .create()
        }

        DistLayerError::ChecksumVerification {
            url,
            expected,
            actual,
        } => {
            let url = style::url(url);
            let expected = style::value(expected);
            let actual = style::value(actual);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header("Node.js distribution checksum verification failed")
                .body(formatdoc! {"
                    An error occurred while verifying the checksum of the Node.js distribution from \
                    {url}. This error can occur due to an issue with the upstream Node.js download server.

                    Checksum:
                    - Expected: {expected}
                    - Actual: {actual}    
                "})
                .create()
        }

        DistLayerError::ReadTempFile { path, source } => {
            let path = file_value(path);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header("Failed to read downloaded Node.js distribution")
                .body(formatdoc! {"
                    An unexpected I/O error occurred while trying to read the downloaded Node.js \
                    distribution from {path}.
                "})
                .debug_info(source.to_string())
                .create()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use heroku_nodejs_utils::package_json::PackageJsonError;
    use insta::{assert_snapshot, with_settings};
    use test_support::test_name;

    #[test]
    fn test_inventory_parse_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::InventoryParse(
            create_toml_error(),
        ));
    }

    #[test]
    fn test_package_json_access_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::PackageJson(
            PackageJsonError::AccessError(create_io_error("test I/O error blah")),
        ));
    }
    #[test]
    fn test_package_json_parse_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::PackageJson(
            PackageJsonError::ParseError(create_json_error()),
        ));
    }

    #[test]
    fn test_unknown_version_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::UnknownVersion(
            "0.0.0".to_string(),
        ));
    }

    #[test]
    fn test_dist_layer_temp_file_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::TempFile(create_io_error("Disk full")),
        ));
    }

    #[test]
    fn test_dist_layer_download_read_error() {
        let url = "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
            .to_string();
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::Download {
                src_url: url.clone(),
                dst_path: "/tmp/some-temp-file".into(),
                source: DownloadError::Request(url, create_reqwest_error()),
            },
        ));
    }

    #[test]
    fn test_dist_layer_download_write_error() {
        let url = "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
            .to_string();
        let path = "/tmp/some-temp-file";
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::Download {
                src_url: url.clone(),
                dst_path: path.into(),
                source: DownloadError::WriteFile(path.into(), url, create_io_error("Disk full")),
            },
        ));
    }

    #[test]
    fn test_dist_layer_untar_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::Untar {
                src_path: "/tmp/some-temp-file".into(),
                dst_path: "/layers/buildpack/some-layer-dir".into(),
                source: create_io_error("permission denied"),
            },
        ));
    }

    #[test]
    fn test_dist_layer_tarball_prefix_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::TarballPrefix(
                "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
                    .to_string(),
            ),
        ));
    }

    #[test]
    fn test_dist_layer_installation_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::Installation {
                src_path: "/tmp/path-to-src".into(),
                dst_path: "/layers/buildpack/path-to-dst".into(),
                source: create_io_error("unexpected end of file"),
            },
        ));
    }

    #[test]
    fn test_dist_layer_checksum_verification_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::ChecksumVerification {
                url: "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
                    .into(),
                expected: "d41d8cd98f00b204e9800998ecf8427e".into(),
                actual: "e62ff0123a74adfc6903d59a449cbdb0".into(),
            },
        ));
    }

    #[test]
    fn test_dist_layer_read_temp_file_error() {
        assert_error_snapshot(NodeJsEngineBuildpackError::DistLayer(
            DistLayerError::ReadTempFile {
                path: "/tmp/path-to-src".into(),
                source: create_io_error("read-only filesystem or storage medium"),
            },
        ));
    }

    fn assert_error_snapshot(error: impl Into<NodeJsEngineBuildpackError>) {
        let error_message = strip_ansi(on_nodejs_engine_buildpack_error(error.into()).to_string());
        let test_name = format!(
            "errors__{}",
            test_name()
                .split("::")
                .last()
                .unwrap()
                .trim_start_matches("test")
        );
        with_settings!({
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            assert_snapshot!(test_name, error_message);
        });
    }

    fn create_io_error(text: &str) -> std::io::Error {
        std::io::Error::other(text)
    }

    fn create_toml_error() -> toml::de::Error {
        toml::from_str::<toml::Table>("[[artifacts").unwrap_err()
    }

    fn create_json_error() -> serde_json::error::Error {
        serde_json::from_str::<serde_json::Value>(r#"{\n  "name":\n}"#).unwrap_err()
    }

    fn create_reqwest_error() -> reqwest_middleware::Error {
        async_runtime().block_on(async {
            reqwest_middleware::Error::Reqwest(
                reqwest::get("https://test/error").await.unwrap_err(),
            )
        })
    }

    fn async_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
    }
}
