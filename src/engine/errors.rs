use super::install_node::DistLayerError;
use super::main::NodeJsEngineBuildpackError;
use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use bullet_stream::style;
use indoc::formatdoc;

pub(crate) fn on_nodejs_engine_error(error: NodeJsEngineBuildpackError) -> ErrorMessage {
    match error {
        NodeJsEngineBuildpackError::DistLayer(e) => on_dist_layer_error(e),
    }
}

#[allow(clippy::too_many_lines)]
fn on_dist_layer_error(error: DistLayerError) -> ErrorMessage {
    match error {
        DistLayerError::Download { src_url, source } => {
            let nodejs_status_url = style::url("https://status.nodejs.org/");
            let src_url = style::url(src_url);
            error_message()
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
                .create()
        }

        DistLayerError::Installation {
            url,
            dst_path,
            source,
        } => {
            let url = style::url(url);
            let dst_path = file_value(dst_path);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header("Failed to copy Node.js distribution contents")
                .body(formatdoc! {"
                    An unexpected I/O error occurred while copying the contents of the Node.js \
                    distribution from {url} to the installation directory at {dst_path}.
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use insta::{assert_snapshot, with_settings};
    use test_support::test_name;

    #[test]
    fn test_dist_layer_download_error() {
        let url = "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
            .to_string();
        assert_error_snapshot(DistLayerError::Download {
            src_url: url.clone(),
            source: crate::utils::http::Error::Request(url, create_reqwest_error()),
        });
    }

    #[test]
    fn test_dist_layer_installation_error() {
        assert_error_snapshot(DistLayerError::Installation {
            url: "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
                .into(),
            dst_path: "/layers/buildpack/path-to-dst".into(),
            source: create_io_error("unexpected end of file"),
        });
    }

    #[test]
    fn test_dist_layer_checksum_verification_error() {
        assert_error_snapshot(DistLayerError::ChecksumVerification {
            url: "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
                .into(),
            expected: "d41d8cd98f00b204e9800998ecf8427e".into(),
            actual: "e62ff0123a74adfc6903d59a449cbdb0".into(),
        });
    }

    fn assert_error_snapshot(error: impl Into<NodeJsEngineBuildpackError>) {
        let error_message = strip_ansi(on_nodejs_engine_error(error.into()).to_string());
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
