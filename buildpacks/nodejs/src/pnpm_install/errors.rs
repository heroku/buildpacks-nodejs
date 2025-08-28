use super::cmd::PnpmVersionError;
use super::main::PnpmInstallBuildpackError;
use bullet_stream::style;
use heroku_nodejs_utils::buildplan::{
    NodeBuildScriptsMetadataError, NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::error_handling::{
    error_message, file_value, on_package_json_error, ErrorMessage, ErrorType, SuggestRetryBuild,
    SuggestSubmitIssue, BUILDPACK_NAME,
};
use indoc::formatdoc;

#[allow(clippy::too_many_lines)]
pub(crate) fn on_pnpm_install_buildpack_error(error: PnpmInstallBuildpackError) -> ErrorMessage {
    match error {
        PnpmInstallBuildpackError::BuildScript(e) => {
            let build_script = style::value(e.name());
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
                .debug_info(e.to_string())
                .create()
        }

        PnpmInstallBuildpackError::PackageJson(e) => {
            on_package_json_error(e)
        }

        PnpmInstallBuildpackError::PnpmInstall(e) => {
            let pnpm_install = style::value(e.name());
            error_message()
                .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header("Failed to install Node modules")
                .body(formatdoc! { "
                    The {BUILDPACK_NAME} uses the command {pnpm_install} to install your Node modules. This command \
                    failed and the buildpack cannot continue. This error can occur due to an unstable network connection. \
                    See the log output above for more information.
        
                    Suggestions:
                    - Ensure that this command runs locally without error (exit status = 0).
                    - Check the status of the upstream Node module repository service at https://status.npmjs.org/
                " })
                .debug_info(e.to_string())
                .create()
        }

        PnpmInstallBuildpackError::PnpmSetStoreDir(e) => {
            error_message()
                .error_type(ErrorType::Internal)
                .header("Failed to configure pnpm store dir")
                .body(formatdoc! { "
                    An unexpected error occurred while configuring the store directory for pnpm. This is the location \
                    on disk where pnpm saves all packages.
                " })
                .debug_info(e.to_string())
                .create()
        }

        PnpmInstallBuildpackError::PnpmSetVirtualStoreDir(e) => {
            error_message()
                .error_type(ErrorType::Internal)
                .header("Failed to configure pnpm virtual store dir")
                .body(formatdoc! { "
                    An unexpected error occurred while configuring the store directory for pnpm. This is the directory \
                    where pnpm links all installed packages from the store.
                " })
                .debug_info(e.to_string())
                .create()
        }

        PnpmInstallBuildpackError::PnpmStorePrune(e) => {
            error_message()
                .error_type(ErrorType::Internal)
                .header("Failed to prune packages from the store directory")
                .body(formatdoc! { "
                    The {BUILDPACK_NAME} periodically cleans up the store directory to remove \
                    packages that are no longer in use from the cache. An unexpected error occurred \
                    during this operation.
                " })
                .debug_info(e.to_string())
                .create()
        }

        PnpmInstallBuildpackError::CreateDirectory(path, e) => {
            let path = file_value(path);
            error_message()
                .error_type(ErrorType::Internal)
                .header("Failed to create directory")
                .body(formatdoc! { "
                    An unexpected I/O error occurred while creating the directory at {path}.
                " })
                .debug_info(e.to_string())
                .create()
        }

        PnpmInstallBuildpackError::CreateSymlink { from, to, source } => {
            let from = file_value(from);
            let to = file_value(to);
            error_message()
                .error_type(ErrorType::Internal)
                .header("Failed to create symlink")
                .body(formatdoc! { "
                    An unexpected I/O error occurred while create a symlink from {from} to {to}.
                " })
                .debug_info(source.to_string())
                .create()
        }

        PnpmInstallBuildpackError::NodeBuildScriptsMetadata(error) => {
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

        PnpmInstallBuildpackError::PruneDevDependencies(error) => error_message()
            .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
            .header("Failed to prune pnpm dev dependencies")
            .body(formatdoc! { "
                The {BUILDPACK_NAME} uses the command {pnpm_prune} to remove your dev dependencies from the production environment. This command \
                failed and the buildpack cannot continue. See the log output above for more information.

                Suggestions:
                - Ensure that this command runs locally without error (exit status = 0).
            ", pnpm_prune = style::value(error.name()) })
            .debug_info(error.to_string())
            .create(),

        PnpmInstallBuildpackError::PnpmVersion(error) => {
                match error {
                    PnpmVersionError::Command(e) => error_message()
                        .error_type(ErrorType::Internal)
                        .header("Failed to determine pnpm version")
                        .body(formatdoc! { "
                            An unexpected error occurred while attempting to determine the current pnpm version \
                            from the system.
                        " })
                        .debug_info(e.to_string())
                        .create(),

                    PnpmVersionError::Parse(stdout, e) => error_message()
                        .error_type(ErrorType::Internal)
                        .header("Failed to parse pnpm version")
                        .body(formatdoc! { "
                            An unexpected error occurred while parsing pnpm version information from '{stdout}'.
                        " })
                        .debug_info(e.to_string())
                        .create(),
                }
        },

        PnpmInstallBuildpackError::Config(error) => error.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use fun_run::{CmdError, CommandWithName};
    use heroku_nodejs_utils::package_json::PackageJsonError;
    use heroku_nodejs_utils::vrs::Version;
    use insta::{assert_snapshot, with_settings};
    use std::io;
    use std::process::Command;
    use test_support::test_name;

    #[test]
    fn test_pnpm_install_node_build_scripts_metadata_error_for_invalid_enabled_value() {
        assert_error_snapshot(PnpmInstallBuildpackError::NodeBuildScriptsMetadata(
            NodeBuildScriptsMetadataError::InvalidEnabledValue(toml::value::Value::String(
                "test".to_string(),
            )),
        ));
    }

    #[test]
    fn test_pnpm_install_node_build_scripts_metadata_error_for_invalid_skip_pruning_value() {
        assert_error_snapshot(PnpmInstallBuildpackError::NodeBuildScriptsMetadata(
            NodeBuildScriptsMetadataError::InvalidSkipPruningValue(toml::value::Value::String(
                "test".to_string(),
            )),
        ));
    }

    #[test]
    fn test_pnpm_install_set_store_dir_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PnpmSetStoreDir(
            create_cmd_error("pnpm config set store-dir /some/dir --global"),
        ));
    }

    #[test]
    fn test_pnpm_install_set_virtual_store_dir_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PnpmSetVirtualStoreDir(
            create_cmd_error("pnpm config set virtual-store-dir /some/dir --global"),
        ));
    }

    #[test]
    fn test_pnpm_install_pnpm_install_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PnpmInstall(create_cmd_error(
            "pnpm install",
        )));
    }

    #[test]
    fn test_pnpm_install_build_script_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::BuildScript(create_cmd_error(
            "pnpm run build",
        )));
    }

    #[test]
    fn test_pnpm_install_package_json_access_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PackageJson(
            PackageJsonError::AccessError(create_io_error("test I/O error blah")),
        ));
    }

    #[test]
    fn test_pnpm_install_package_json_parse_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PackageJson(
            PackageJsonError::ParseError(create_json_error()),
        ));
    }

    #[test]
    fn test_pnpm_install_pnpm_store_prune_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PnpmStorePrune(create_cmd_error(
            "pnpm prune",
        )));
    }

    #[test]
    fn test_pnpm_install_create_directory_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::CreateDirectory(
            "/layers/pnpm_install/dir".into(),
            create_io_error("Insufficient permissions"),
        ));
    }

    #[test]
    fn test_pnpm_install_create_symlink_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::CreateSymlink {
            from: "/app/node_modules/.pnpm".into(),
            to: "/layers/pnpm_install/dir".into(),
            source: create_io_error("Target directory does not exist"),
        });
    }

    #[test]
    fn test_pnpm_install_prune_dev_dependencies_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PruneDevDependencies(
            create_cmd_error("pnpm prune --prod --ignore-scripts"),
        ));
    }

    #[test]
    fn test_pnpm_install_pnpm_version_command_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PnpmVersion(
            PnpmVersionError::Command(create_cmd_error("pnpm --version")),
        ));
    }

    #[test]
    fn test_pnpm_install_pnpm_version_parse_error() {
        assert_error_snapshot(PnpmInstallBuildpackError::PnpmVersion(
            PnpmVersionError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    fn assert_error_snapshot(error: impl Into<PnpmInstallBuildpackError>) {
        let error_message = strip_ansi(on_pnpm_install_buildpack_error(error.into()).to_string());
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
