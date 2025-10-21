use super::cmd::YarnVersionError;
use super::main::YarnBuildpackError;
use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    BUILDPACK_NAME, ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message,
    on_package_json_error,
};
use bullet_stream::style;
use fun_run::CmdError;
use indoc::formatdoc;

pub(crate) fn on_yarn_error(error: YarnBuildpackError) -> ErrorMessage {
    match error {
        YarnBuildpackError::PackageJson(e) => on_package_json_error(e),
        YarnBuildpackError::YarnVersionDetect(e) => on_yarn_version_detect_error(&e),
        YarnBuildpackError::YarnVersionUnsupported(version) => {
            on_yarn_version_unsupported_error(version)
        }
        YarnBuildpackError::PruneYarnDevDependencies(e) => on_prune_dev_dependencies_error(&e),
        YarnBuildpackError::InstallPrunePluginError(e) => on_install_prune_plugin_error(&e),
    }
}

fn on_yarn_version_detect_error(error: &YarnVersionError) -> ErrorMessage {
    match error {
        YarnVersionError::Command(e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to determine Yarn version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current Yarn version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        YarnVersionError::Parse(stdout, e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to parse npm version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing Yarn version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create()
    }
}

fn on_yarn_version_unsupported_error(version: u64) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::No,
            SuggestSubmitIssue::Yes,
        ))
        .header("Unsupported Yarn version")
        .body(formatdoc! {"
            The {BUILDPACK_NAME} does not support Yarn version {version}.

            Suggestions:
            - Update your package.json to specify a supported Yarn version.
        "})
        .create()
}

fn on_prune_dev_dependencies_error(error: &CmdError) -> ErrorMessage {
    let yarn_prune = style::value(error.name());
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header("Failed to prune Yarn dev dependencies")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} uses the command {yarn_prune} to remove your dev dependencies from the production environment. This command \
            failed and the buildpack cannot continue. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
        " })
        .debug_info(error.to_string())
        .create()
}

fn on_install_prune_plugin_error(error: &std::io::Error) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to install Yarn plugin for pruning")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} uses a custom plugin for Yarn to handle pruning \
            of dev dependencies. An unexpected error was encountered while trying to install it.
        " })
        .debug_info(error.to_string())
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::package_json::PackageJsonError;
    use crate::utils::vrs::{Version, VersionError};
    use bullet_stream::strip_ansi;
    use fun_run::{CmdError, CommandWithName};
    use insta::{assert_snapshot, with_settings};
    use std::process::Command;
    use test_support::test_name;

    #[test]
    fn test_yarn_package_json_access_error() {
        assert_error_snapshot(YarnBuildpackError::PackageJson(
            PackageJsonError::AccessError(create_io_error("test I/O error blah")),
        ));
    }

    #[test]
    fn test_yarn_package_json_parse_error() {
        assert_error_snapshot(YarnBuildpackError::PackageJson(
            PackageJsonError::ParseError(create_json_error()),
        ));
    }

    #[test]
    fn test_yarn_version_detect_yarn_version_command_error() {
        assert_error_snapshot(YarnBuildpackError::YarnVersionDetect(
            YarnVersionError::Command(create_cmd_error("yarn --version")),
        ));
    }

    #[test]
    fn test_yarn_version_detect_yarn_version_parse_error() {
        assert_error_snapshot(YarnBuildpackError::YarnVersionDetect(
            YarnVersionError::Parse("not.a.version".to_string(), create_version_error()),
        ));
    }

    #[test]
    fn test_yarn_version_unsupported_error() {
        assert_error_snapshot(YarnBuildpackError::YarnVersionUnsupported(0));
    }

    #[test]
    fn test_yarn_prune_dev_dependencies_error() {
        assert_error_snapshot(YarnBuildpackError::PruneYarnDevDependencies(
            create_cmd_error("yarn heroku prune"),
        ));
    }

    #[test]
    fn test_yarn_install_prune_plugin_error() {
        assert_error_snapshot(YarnBuildpackError::InstallPrunePluginError(
            create_io_error("Out of disk space"),
        ));
    }

    fn assert_error_snapshot(error: impl Into<YarnBuildpackError>) {
        let error_message = strip_ansi(on_yarn_error(error.into()).to_string());
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
        Version::parse("not.a.version").unwrap_err()
    }

    fn create_json_error() -> serde_json::error::Error {
        serde_json::from_str::<serde_json::Value>(r#"{\n  "name":\n}"#).unwrap_err()
    }
}
