use super::cmd::CorepackVersionError;
use super::main::CorepackBuildpackError;
use bullet_stream::style;
use fun_run::CmdError;
use heroku_nodejs_utils::error_handling::error_message_builder::SetIssuesUrl;
use heroku_nodejs_utils::error_handling::{
    file_value, on_framework_error, on_package_json_error, ErrorMessage, ErrorMessageBuilder,
    ErrorType, SuggestRetryBuild, SuggestSubmitIssue,
};
use indoc::formatdoc;
use libcnb::Error;
use std::path::Path;

const BUILDPACK_NAME: &str = "Heroku Node.js Corepack";

const ISSUES_URL: &str = "https://github.com/heroku/buildpacks-nodejs/issues";

pub(crate) fn on_error(error: libcnb::Error<CorepackBuildpackError>) -> ErrorMessage {
    match error {
        Error::BuildpackError(e) => on_buildpack_error(e),
        e => on_framework_error(BUILDPACK_NAME, ISSUES_URL, &e),
    }
}

// Wraps the error_message() builder to preset the issues_url field
fn error_message() -> ErrorMessageBuilder<SetIssuesUrl> {
    heroku_nodejs_utils::error_handling::error_message().issues_url(ISSUES_URL.to_string())
}

fn on_buildpack_error(error: CorepackBuildpackError) -> ErrorMessage {
    match error {
        CorepackBuildpackError::CorepackEnable(e) => on_corepack_enable_error(&e),
        CorepackBuildpackError::CorepackPrepare(e) => on_corepack_prepare_error(&e),
        CorepackBuildpackError::CorepackVersion(e) => on_corepack_version_error(&e),
        CorepackBuildpackError::CreateBinDirectory(path, e) => {
            on_create_bin_directory_error(&path, &e)
        }
        CorepackBuildpackError::CreateCacheDirectory(path, e) => {
            on_create_cache_directory_error(&path, &e)
        }
        CorepackBuildpackError::PackageJson(e) => {
            on_package_json_error(BUILDPACK_NAME, ISSUES_URL, e)
        }
        CorepackBuildpackError::PackageManagerMissing => on_package_manager_missing_error(),
    }
}

fn on_corepack_enable_error(error: &CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to enable Corepack")
        .body(formatdoc! {
            "Unable to install corepack shims via `corepack enable`."
        })
        .debug_info(error.to_string())
        .create()
}

fn on_corepack_prepare_error(error: &CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to prepare Corepack")
        .body(formatdoc! {
            "Unable to download package manager via `corepack prepare`."
        })
        .debug_info(error.to_string())
        .create()
}

fn on_corepack_version_error(error: &CorepackVersionError) -> ErrorMessage {
    match error {
        CorepackVersionError::Parse(e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to parse Corepack version")
            .body(formatdoc! {
                "Unexpected version output returned from `corepack --version`."
            })
            .debug_info(e.to_string())
            .create(),

        CorepackVersionError::Command(e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to determine Corepack version")
            .body(formatdoc! {
                "Unexpected error executing `corepack --version`."
            })
            .debug_info(e.to_string())
            .create(),
    }
}

fn on_create_bin_directory_error(path: &Path, error: &std::io::Error) -> ErrorMessage {
    let path = file_value(path);
    error_message()
        .error_type(ErrorType::Internal)
        .header("Unable to create Corepack shim bin directory")
        .body(formatdoc! { "
            An unexpected error occurred while creating the directory at {path}.
             
            This directory contains the binaries used as shims for Corepack functionality."
        })
        .debug_info(error.to_string())
        .create()
}

fn on_create_cache_directory_error(path: &Path, error: &std::io::Error) -> ErrorMessage {
    let path = file_value(path);
    error_message()
        .error_type(ErrorType::Internal)
        .header("Unable to create Corepack package manager cache directory")
        .body(formatdoc! { "
            An unexpected error occurred while creating the directory at {path}.
             
            This directory contains the cache to store Corepack downloaded package managers."
        })
        .debug_info(error.to_string())
        .create()
}

fn on_package_manager_missing_error() -> ErrorMessage {
    let package_manager = style::value("packageManager");
    let package_json = style::value("package.json");
    let corepack_usage_url =
        style::url("https://github.com/nodejs/corepack?tab=readme-ov-file#when-authoring-packages");
    error_message()
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::No,
            SuggestSubmitIssue::No,
        ))
        .header("Invalid Corepack packageManager")
        .body(formatdoc! { "
            There was an error reading the {package_manager} field from {package_json}. 

            Suggestions:
            - Ensure the field value matches Corepack usage format ({corepack_usage_url})
        "})
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use fun_run::CommandWithName;
    use heroku_nodejs_utils::package_json::PackageJsonError;
    use heroku_nodejs_utils::vrs::{Version, VersionError};
    use insta::{assert_snapshot, with_settings};
    use std::path::PathBuf;
    use std::process::Command;
    use test_support::test_name;

    #[test]
    fn test_corepack_enable_error() {
        assert_error_snapshot(CorepackBuildpackError::CorepackEnable(create_cmd_error(
            "corepack --enable",
        )));
    }
    #[test]
    fn test_corepack_prepare_error() {
        assert_error_snapshot(CorepackBuildpackError::CorepackPrepare(create_cmd_error(
            "corepack prepare pkg-mgr@1.2.3",
        )));
    }

    #[test]
    fn test_corepack_version_parse_error() {
        assert_error_snapshot(CorepackBuildpackError::CorepackVersion(
            CorepackVersionError::Parse(create_version_error()),
        ));
    }

    #[test]
    fn test_corepack_version_command_error() {
        assert_error_snapshot(CorepackBuildpackError::CorepackVersion(
            CorepackVersionError::Command(create_cmd_error("corepack --version")),
        ));
    }

    #[test]
    fn test_corepack_create_bin_directory_error() {
        assert_error_snapshot(CorepackBuildpackError::CreateBinDirectory(
            PathBuf::from("/layers/corepack/shims/bin"),
            create_io_error("Disk full"),
        ));
    }

    #[test]
    fn test_corepack_create_cache_directory_error() {
        assert_error_snapshot(CorepackBuildpackError::CreateCacheDirectory(
            PathBuf::from("/layers/corepack/mgr/cache"),
            create_io_error("Disk full"),
        ));
    }

    #[test]
    fn test_package_json_access_error() {
        assert_error_snapshot(CorepackBuildpackError::PackageJson(
            PackageJsonError::AccessError(create_io_error("test I/O error blah")),
        ));
    }

    #[test]
    fn test_package_json_parse_error() {
        assert_error_snapshot(CorepackBuildpackError::PackageJson(
            PackageJsonError::ParseError(create_json_error()),
        ));
    }

    #[test]
    fn test_corepack_package_manager_missing_error() {
        assert_error_snapshot(CorepackBuildpackError::PackageManagerMissing);
    }

    fn assert_error_snapshot(error: impl Into<Error<CorepackBuildpackError>>) {
        let error_message = strip_ansi(on_error(error.into()).to_string());
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

    fn create_cmd_error(command: impl Into<String>) -> CmdError {
        Command::new("false")
            .named(command.into())
            .named_output()
            .unwrap_err()
    }

    fn create_version_error() -> VersionError {
        Version::parse("not.a-version").unwrap_err()
    }

    fn create_json_error() -> serde_json::error::Error {
        serde_json::from_str::<serde_json::Value>(r#"{\n  "name":\n}"#).unwrap_err()
    }
}
