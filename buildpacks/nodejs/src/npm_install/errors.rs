use super::main::NpmInstallBuildpackError;
use super::npm;
use bullet_stream::style;
use fun_run::CmdError;
use heroku_nodejs_utils::application;
use heroku_nodejs_utils::buildplan::{
    NodeBuildScriptsMetadataError, NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::error_handling::{
    error_message, on_package_json_error, ErrorMessage, ErrorType, SuggestRetryBuild,
    SuggestSubmitIssue, BUILDPACK_NAME,
};
use indoc::formatdoc;
use std::io;

pub(crate) fn on_npm_install_buildpack_error(error: NpmInstallBuildpackError) -> ErrorMessage {
    match error {
        NpmInstallBuildpackError::Application(e) => on_application_error(&e),
        NpmInstallBuildpackError::BuildScript(e) => on_build_script_error(&e),
        NpmInstallBuildpackError::Detect(e) => on_detect_error(&e),
        NpmInstallBuildpackError::NodeBuildScriptsMetadata(e) => {
            on_node_build_scripts_metadata_error(e)
        }
        NpmInstallBuildpackError::NpmInstall(e) => on_npm_install_error(&e),
        NpmInstallBuildpackError::NpmSetCacheDir(e) => on_set_cache_dir_error(&e),
        NpmInstallBuildpackError::NpmVersion(e) => on_npm_version_error(e),
        NpmInstallBuildpackError::PackageJson(e) => on_package_json_error(e),
        NpmInstallBuildpackError::PruneDevDependencies(e) => on_prune_dev_dependencies_error(&e),
        NpmInstallBuildpackError::Config(e) => e.into(),
    }
}

fn on_node_build_scripts_metadata_error(error: NodeBuildScriptsMetadataError) -> ErrorMessage {
    let requires_metadata = style::value("[requires.metadata]");
    let buildplan_name = style::value(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME);

    match error {
        NodeBuildScriptsMetadataError::InvalidEnabledValue(value) => error_message()
            .error_type(ErrorType::UserFacing(
                SuggestRetryBuild::No,
                SuggestSubmitIssue::Yes,
            ))
            .header("Invalid build plan metadata")
            .body(formatdoc! { "
                A participating buildpack has set invalid {requires_metadata} for the build plan \
                named {buildplan_name}.
                
                Expected metadata format:
                [requires.metadata]
                enabled = <bool>
                skip_pruning = <bool>
                
                But configured with:
                enabled = {value} <{value_type}>     
            ", value_type = value.type_str() })
            .create(),

        NodeBuildScriptsMetadataError::InvalidSkipPruningValue(value) => error_message()
            .error_type(ErrorType::UserFacing(
                SuggestRetryBuild::No,
                SuggestSubmitIssue::Yes,
            ))
            .header("Invalid build plan metadata")
            .body(formatdoc! { "
                A participating buildpack has set invalid {requires_metadata} for the build plan \
                named {buildplan_name}.
                
                Expected metadata format:
                [requires.metadata]
                enabled = <bool>
                skip_pruning = <bool>
                
                But configured with:
                skip_pruning = {value} <{value_type}>     
            ", value_type = value.type_str() })
            .create(),
    }
}

fn on_set_cache_dir_error(error: &CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to set the npm cache directory")
        .body("An unexpected error occurred while setting the npm cache directory.")
        .debug_info(error.to_string())
        .create()
}

fn on_npm_version_error(error: npm::VersionError) -> ErrorMessage {
    match error {
        npm::VersionError::Command(e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to determine npm version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current npm version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        npm::VersionError::Parse(stdout, e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to parse npm version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing npm version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

fn on_npm_install_error(error: &CmdError) -> ErrorMessage {
    let npm_install = style::value(error.name());
    error_message()
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header("Failed to install Node modules")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} buildpack uses the command {npm_install} to install your Node modules. This command \
            failed and the buildpack cannot continue. This error can occur due to an unstable network connection. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
            - Check the status of the upstream Node module repository service at https://status.npmjs.org/
        " })
        .debug_info(error.to_string())
        .create()
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
            The {BUILDPACK_NAME} buildpack allows customization of the build process by executing the following scripts \
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

fn on_application_error(error: &application::Error) -> ErrorMessage {
    match error {
        application::Error::MissingLockfile => error_message()
            .error_type(ErrorType::UserFacing(
                SuggestRetryBuild::No,
                SuggestSubmitIssue::No,
            ))
            .header("Missing lockfile")
            .body(error.to_string())
            .create(),

        application::Error::MultipleLockfiles(_) => error_message()
            .error_type(ErrorType::UserFacing(
                SuggestRetryBuild::No,
                SuggestSubmitIssue::No,
            ))
            .header("Multiple lockfiles detected")
            .body(error.to_string())
            .create(),
    }
}

fn on_detect_error(error: &io::Error) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Unable to complete buildpack detection")
        .body(formatdoc! { "
            An unexpected error occurred while determining if the {BUILDPACK_NAME} buildpack should be \
            run for this application. See the log output above for more information.
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
            The {BUILDPACK_NAME} buildpack uses the command {npm_prune} to remove your dev dependencies from the production environment. This command \
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
    use bullet_stream::strip_ansi;
    use fun_run::CommandWithName;
    use heroku_nodejs_utils::package_json::PackageJsonError;
    use heroku_nodejs_utils::package_manager::PackageManager;
    use heroku_nodejs_utils::vrs::Version;
    use insta::{assert_snapshot, with_settings};
    use libcnb::Error;
    use std::process::Command;
    use test_support::test_name;

    #[test]
    fn test_npm_install_application_error_missing_lockfile() {
        assert_error_snapshot(NpmInstallBuildpackError::Application(
            application::Error::MissingLockfile,
        ));
    }

    #[test]
    fn test_npm_install_application_error_multiple_lockfiles() {
        assert_error_snapshot(NpmInstallBuildpackError::Application(
            application::Error::MultipleLockfiles(vec![
                PackageManager::Npm,
                PackageManager::Yarn,
                PackageManager::Pnpm,
            ]),
        ));
    }

    #[test]
    fn test_npm_install_node_build_scripts_metadata_error_for_invalid_enabled_value() {
        assert_error_snapshot(NpmInstallBuildpackError::NodeBuildScriptsMetadata(
            NodeBuildScriptsMetadataError::InvalidEnabledValue(toml::value::Value::String(
                "test".to_string(),
            )),
        ));
    }

    #[test]
    fn test_npm_install_node_build_scripts_metadata_error_for_invalid_skip_pruning_value() {
        assert_error_snapshot(NpmInstallBuildpackError::NodeBuildScriptsMetadata(
            NodeBuildScriptsMetadataError::InvalidSkipPruningValue(toml::value::Value::String(
                "test".to_string(),
            )),
        ));
    }

    #[test]
    fn test_npm_install_set_cache_dir_error() {
        assert_error_snapshot(NpmInstallBuildpackError::NpmSetCacheDir(create_cmd_error(
            "npm config set cache /some/dir --global",
        )));
    }

    #[test]
    fn test_npm_install_npm_version_command_error() {
        assert_error_snapshot(NpmInstallBuildpackError::NpmVersion(
            npm::VersionError::Command(create_cmd_error("npm --version")),
        ));
    }

    #[test]
    fn test_npm_install_npm_version_parse_error() {
        assert_error_snapshot(NpmInstallBuildpackError::NpmVersion(
            npm::VersionError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    #[test]
    fn test_npm_install_npm_install_error() {
        assert_error_snapshot(NpmInstallBuildpackError::NpmInstall(create_cmd_error(
            "npm install",
        )));
    }

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
    fn test_npm_install_detect_error() {
        assert_error_snapshot(NpmInstallBuildpackError::Detect(create_io_error(
            "test I/O error blah",
        )));
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

    fn create_io_error(text: &str) -> io::Error {
        io::Error::other(text)
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
