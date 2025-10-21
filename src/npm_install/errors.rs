use super::main::NpmInstallBuildpackError;
use crate::utils::error_handling::{
    BUILDPACK_NAME, ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message,
    on_package_json_error,
};
use bullet_stream::style;
use fun_run::CmdError;
use indoc::formatdoc;

pub(crate) fn on_npm_install_buildpack_error(error: NpmInstallBuildpackError) -> ErrorMessage {
    match error {
        NpmInstallBuildpackError::BuildScript(e) => on_build_script_error(&e),
        NpmInstallBuildpackError::PackageJson(e) => on_package_json_error(e),
        NpmInstallBuildpackError::PruneDevDependencies(e) => on_prune_dev_dependencies_error(&e),
    }
}

fn on_build_script_error(error: &CmdError) -> ErrorMessage {
    let build_script = style::value(error.name());
    let package_json = style::value("package.json");
    let heroku_prebuild = style::value("heroku-prebuild");
    let heroku_build = style::value("heroku-build");
    let build = style::value("build");
    let heroku_postbuild = style::value("heroku-postbuild");
    error_message()
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
        .header("Failed to execute build script")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} allows customization of the build process by executing the following scripts \
            if they are defined in {package_json}:
            - {heroku_prebuild} 
            - {heroku_build} or {build} 
            - {heroku_postbuild}

            An unexpected error occurred while executing {build_script}. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error.
        "})
        .debug_info(error.to_string())
        .create()
}

fn on_prune_dev_dependencies_error(error: &CmdError) -> ErrorMessage {
    let npm_prune = style::value(error.name());
    error_message()
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header("Failed to prune npm dev dependencies")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} uses the command {npm_prune} to remove your dev dependencies from the production environment. This command \
            failed and the buildpack cannot continue. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
        " })
        .debug_info(error.to_string())
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::package_json::PackageJsonError;
    use bullet_stream::strip_ansi;
    use fun_run::CommandWithName;
    use insta::{assert_snapshot, with_settings};
    use std::process::Command;
    use test_support::test_name;

    #[test]
    fn test_npm_install_build_script_error() {
        assert_error_snapshot(NpmInstallBuildpackError::BuildScript(create_cmd_error(
            "npm run build",
        )));
    }

    #[test]
    fn test_npm_install_package_json_access_error() {
        assert_error_snapshot(NpmInstallBuildpackError::PackageJson(
            PackageJsonError::AccessError(create_io_error("test I/O error blah")),
        ));
    }

    #[test]
    fn test_npm_install_package_json_parse_error() {
        assert_error_snapshot(NpmInstallBuildpackError::PackageJson(
            PackageJsonError::ParseError(create_json_error()),
        ));
    }

    #[test]
    fn test_npm_install_prune_dev_dependencies_error() {
        assert_error_snapshot(NpmInstallBuildpackError::PruneDevDependencies(
            create_cmd_error("npm prune --production"),
        ));
    }

    fn assert_error_snapshot(error: impl Into<NpmInstallBuildpackError>) {
        let error_message = strip_ansi(on_npm_install_buildpack_error(error.into()).to_string());
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

    fn create_json_error() -> serde_json::error::Error {
        serde_json::from_str::<serde_json::Value>(r#"{\n  "name":\n}"#).unwrap_err()
    }

    fn create_cmd_error(command: impl Into<String>) -> CmdError {
        Command::new("false")
            .named(command.into())
            .named_output()
            .unwrap_err()
    }
}
