use crate::cmd::{GetNodeLinkerError, YarnVersionError};
use crate::configure_yarn_cache::DepsLayerError;
use crate::install_yarn::CliLayerError;
use crate::YarnBuildpackError;
use bullet_stream::style;
use fun_run::CmdError;
use heroku_nodejs_utils::buildplan::{
    NodeBuildScriptsMetadataError, NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use heroku_nodejs_utils::error_handling::error_message_builder::SetIssuesUrl;
use heroku_nodejs_utils::error_handling::ErrorType::UserFacing;
use heroku_nodejs_utils::error_handling::{
    file_value, on_framework_error, on_package_json_error, ErrorMessage, ErrorMessageBuilder,
    ErrorType, SharedErrorMessageBuilder, SuggestRetryBuild, SuggestSubmitIssue,
};
use heroku_nodejs_utils::npmjs_org::PackumentLayerError;
use heroku_nodejs_utils::vrs::{Requirement, VersionError};
use indoc::formatdoc;

const BUILDPACK_NAME: &str = "Heroku Node.js Yarn";

const ISSUES_URL: &str = "https://github.com/heroku/buildpacks-nodejs/issues";

pub(crate) fn on_error(error: libcnb::Error<YarnBuildpackError>) -> ErrorMessage {
    match error {
        libcnb::Error::BuildpackError(e) => on_buildpack_error(e),
        e => on_framework_error(BUILDPACK_NAME, ISSUES_URL, &e),
    }
}

// Wraps the error_message() builder to preset the issues_url field
fn error_message() -> ErrorMessageBuilder<SetIssuesUrl> {
    heroku_nodejs_utils::error_handling::error_message().issues_url(ISSUES_URL.to_string())
}

fn on_buildpack_error(error: YarnBuildpackError) -> ErrorMessage {
    match error {
        YarnBuildpackError::BuildScript(e) => on_build_script_error(&e),
        YarnBuildpackError::CliLayer(e) => on_cli_layer_error(&e),
        YarnBuildpackError::DepsLayer(e) => on_deps_layer_error(&e),
        YarnBuildpackError::PackageJson(e) => on_package_json_error(BUILDPACK_NAME, ISSUES_URL, e),
        YarnBuildpackError::YarnCacheGet(e) => on_yarn_cache_get_error(&e),
        YarnBuildpackError::YarnDisableGlobalCache(e) => on_yarn_disable_global_cache_error(&e),
        YarnBuildpackError::YarnInstall(e) => on_yarn_install_error(&e),
        YarnBuildpackError::YarnVersionDetect(e) => on_yarn_version_detect_error(&e),
        YarnBuildpackError::YarnVersionUnsupported(version) => {
            on_yarn_version_unsupported_error(version)
        }
        YarnBuildpackError::YarnVersionResolve(requirement) => {
            on_yarn_version_resolve_error(&requirement)
        }
        YarnBuildpackError::YarnDefaultParse(e) => on_yarn_default_parse_error(&e),
        YarnBuildpackError::NodeBuildScriptsMetadata(e) => on_node_build_scripts_metadata_error(e),
        YarnBuildpackError::FetchYarnPackument(e) => on_fetch_yarn_packument_error(&e),
        YarnBuildpackError::PruneYarnDevDependencies(e) => on_prune_dev_dependencies_error(&e),
        YarnBuildpackError::YarnGetNodeLinker(e) => on_get_node_linker_error(&e),
        YarnBuildpackError::InstallPrunePluginError(e) => on_install_prune_plugin_error(&e),
        YarnBuildpackError::Config(e) => SharedErrorMessageBuilder::from(e)
            .issues_url(ISSUES_URL.to_string())
            .create(),
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

fn on_cli_layer_error(error: &CliLayerError) -> ErrorMessage {
    match error {
        CliLayerError::TempFile(e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to open temporary file")
            .body(formatdoc! {"
                An unexpected I/O error occurred while downloading the Yarn package manager into a \
                temporary directory. 
            " })
            .debug_info(e.to_string())
            .create(),

        CliLayerError::Download(e) => error_message()
            .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
            .header("Failed to download Yarn")
            .body(formatdoc! {"
                    An unexpected error occurred while downloading the Yarn package manager. This error can \
                    occur due to an unstable network connection or an issue with the upstream repository.

                    Suggestions:
                    - Check the npm status page for any ongoing incidents ({npm_status_url})
                ", npm_status_url = style::url("https://status.npmjs.org/") })
            .debug_info(e.to_string())
            .create(),

        CliLayerError::Untar(path, e) => error_message()
            .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
            .header("Failed to extract the downloaded Yarn package file")
            .body(formatdoc! {"
                An unexpected I/O occurred while extracting the contents of the downloaded Yarn package file at {path}.
            ", path = file_value(path) })
            .debug_info(e.to_string())
            .create(),

        CliLayerError::Installation(e) => error_message()
            .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
            .header("Failed to install the downloaded Yarn package")
            .body(formatdoc! {"
                An unexpected error occurred while installing the downloaded Yarn package.
            " })
            .debug_info(e.to_string())
            .create(),

        CliLayerError::Permissions(e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Permissions error for Yarn installation")
            .body(formatdoc! {"
                An unexpected I/O error occurred while setting permissions on the Yarn package manager installation.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

fn on_deps_layer_error(error: &DepsLayerError) -> ErrorMessage {
    match error {
        DepsLayerError::CreateCacheDir(path, e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to create Yarn cache directory")
            .body(formatdoc! {"
                An unexpected I/O error occurred while creating the cache directory at {path} that will be \
                used by the Yarn package manager.
            ", path = file_value(path) })
            .debug_info(e.to_string())
            .create(),

        DepsLayerError::YarnCacheSet(e) => error_message()
            .error_type(ErrorType::Internal)
            .header("Failed to configure Yarn cache directory")
            .body(formatdoc! {"
                An unexpected error occurred while configuring the Yarn cache directory.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

fn on_yarn_cache_get_error(error: &CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to read configured Yarn cache directory")
        .body(formatdoc! {"
            The {BUILDPACK_NAME} buildpack was unable to read the configuration for the Yarn cache directory.
        "})
        .debug_info(error.to_string())
        .create()
}

fn on_yarn_disable_global_cache_error(error: &CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to disable Yarn global cache")
        .body(formatdoc! {"
            The {BUILDPACK_NAME} buildpack was unable to disable the Yarn global cache.
        "})
        .debug_info(error.to_string())
        .create()
}

fn on_yarn_install_error(error: &CmdError) -> ErrorMessage {
    let yarn_install = style::value(error.name());
    error_message()
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::Yes,
        ))
        .header("Failed to install Node modules")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} buildpack uses the command {yarn_install} to install your Node modules. This command \
            failed and the buildpack cannot continue. This error can occur due to an unstable network connection. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
            - Check the status of the upstream Node module repository service at https://status.npmjs.org/
        " })
        .debug_info(error.to_string())
        .create()
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
            The {BUILDPACK_NAME} buildpack does not support Yarn version {version}.

            Suggestions:
            - Update your package.json to specify a supported Yarn version.
        "})
        .create()
}

fn on_yarn_version_resolve_error(requirement: &Requirement) -> ErrorMessage {
    let requested_version = style::value(requirement.to_string());
    let yarn_releases_url = style::url("https://github.com/yarnpkg/berry/releases");
    let inventory_url = style::url("https://github.com/heroku/buildpacks-nodejs/blob/main/buildpacks/nodejs-yarn/inventory.toml");
    let package_json = style::value("package.json");
    let engines_key = style::value("engines.yarn");
    error_message()
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!("Error resolving requested Yarn version {requested_version}"))
        .body(formatdoc! { "
            The requested Yarn version could not be resolved to a known release in this buildpack's \
            inventory of Yarn releases.
            
            Suggestions:
            - Confirm if this is a valid Yarn release at {yarn_releases_url}
            - Check if this buildpack includes the requested Yarn version in its inventory file at {inventory_url}
            - Update the {engines_key} field in {package_json} to a single version or version range that \
            includes a published Yarn version.
        " })
        .create()
}

fn on_yarn_default_parse_error(error: &VersionError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to parse default Yarn version")
        .body(formatdoc! {"
            The {BUILDPACK_NAME} buildpack was unable to parse the default Yarn version.
        "})
        .debug_info(error.to_string())
        .create()
}

fn on_node_build_scripts_metadata_error(error: NodeBuildScriptsMetadataError) -> ErrorMessage {
    let requires_metadata = style::value("[requires.metadata]");
    let buildplan_name = style::value(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME);

    match error {
        NodeBuildScriptsMetadataError::InvalidEnabledValue(value) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
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
            .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
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

fn on_fetch_yarn_packument_error(error: &PackumentLayerError) -> ErrorMessage {
    let yarn = style::value("Yarn");
    let npm_status_url = style::url("https://status.npmjs.org/");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header(format!("Failed to load available {yarn} versions"))
        .body(formatdoc! { "
            An unexpected error occurred while loading the available {yarn} versions. This error can \
            occur due to an unstable network connection or an issue with the npm registry.

            Suggestions:
            - Check the npm status page for any ongoing incidents ({npm_status_url})
        "})
        .debug_info(error.to_string())
        .create()
}

fn on_prune_dev_dependencies_error(error: &CmdError) -> ErrorMessage {
    let yarn_prune = style::value(error.name());
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header("Failed to prune Yarn dev dependencies")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} buildpack uses the command {yarn_prune} to remove your dev dependencies from the production environment. This command \
            failed and the buildpack cannot continue. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
        " })
        .debug_info(error.to_string())
        .create()
}

fn on_get_node_linker_error(error: &GetNodeLinkerError) -> ErrorMessage {
    match error {
        GetNodeLinkerError::Parse(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
            .header("Failed to parse Yarn's nodeLinker configuration")
            .body(formatdoc! { "
                An unexpected value was encountered when trying to read Yarn's nodeLinker configuration.
                Expected - 'pnp', 'node-modules', or 'pnpm'
                Actual   - '{value}'

                Suggestions:
                - Run {check_cmd} locally to check your 'nodeLinker' configuration.
                - Set an explicit 'nodeLinker' configuration in {yarnrc_yml} ({install_modes_url})
                ",
                value = e.0,
                check_cmd = style::command("yarn config get nodeLinker"),
                yarnrc_yml = style::value(".yarnrc.yml"),
                install_modes_url = style::url("https://yarnpkg.com/features/linkers")
            })
            .create(),

        GetNodeLinkerError::Command(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::No))
            .header("Failed to read Yarn's nodeLinker configuration")
            .body(formatdoc! { "
                An unexpected value was encountered when trying to read Yarn's nodeLinker configuration. This \
                configuration is read using the command {read_cmd}.

                Suggestions:
                - Ensure the above command runs locally.
            ", read_cmd = style::command(e.name()) })
            .debug_info(e.to_string()).create()
    }
}

fn on_install_prune_plugin_error(error: &std::io::Error) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to install Yarn plugin for pruning")
        .body(formatdoc! { "
            The {BUILDPACK_NAME} buildpack uses a custom plugin for Yarn to handle pruning \
            of dev dependencies. An unexpected error was encountered while trying to install it.
        " })
        .debug_info(error.to_string())
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yarn::UnknownNodeLinker;
    use bullet_stream::strip_ansi;
    use fun_run::{CmdError, CommandWithName};
    use heroku_nodejs_utils::package_json::PackageJsonError;
    use heroku_nodejs_utils::vrs::Version;
    use insta::{assert_snapshot, with_settings};
    use libcnb::Error;
    use std::process::Command;
    use test_support::test_name;

    #[test]
    fn test_yarn_build_script_error() {
        assert_error_snapshot(YarnBuildpackError::BuildScript(create_cmd_error(
            "yarn run build",
        )));
    }

    #[test]
    fn test_yarn_cli_layer_temp_file_error() {
        assert_error_snapshot(YarnBuildpackError::CliLayer(CliLayerError::TempFile(
            create_io_error("Disk full"),
        )));
    }

    #[test]
    fn test_yarn_cli_layer_download_error() {
        assert_error_snapshot(YarnBuildpackError::CliLayer(CliLayerError::Download(
            heroku_nodejs_utils::http::Error::Request(
                "https://test/error".into(),
                create_reqwest_error(),
            ),
        )));
    }

    #[test]
    fn test_yarn_cli_layer_untar_error() {
        assert_error_snapshot(YarnBuildpackError::CliLayer(CliLayerError::Untar(
            "/layers/yarn/dist".into(),
            create_io_error("Disk full"),
        )));
    }

    #[test]
    fn test_yarn_cli_layer_installation_error() {
        assert_error_snapshot(YarnBuildpackError::CliLayer(CliLayerError::Installation(
            create_io_error("Disk full"),
        )));
    }

    #[test]
    fn test_yarn_cli_layer_permissions_error() {
        assert_error_snapshot(YarnBuildpackError::CliLayer(CliLayerError::Permissions(
            create_io_error("Invalid permissions"),
        )));
    }

    #[test]
    fn test_yarn_deps_layer_create_cache_dir_error() {
        assert_error_snapshot(YarnBuildpackError::DepsLayer(
            DepsLayerError::CreateCacheDir(
                "/layers/yarn/deps/cache".into(),
                create_io_error("Disk full"),
            ),
        ));
    }

    #[test]
    fn test_yarn_deps_layer_yarn_cache_set_error() {
        assert_error_snapshot(YarnBuildpackError::DepsLayer(DepsLayerError::YarnCacheSet(
            create_cmd_error("yarn config set cache-dir /some/dir"),
        )));
    }

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
    fn test_yarn_cache_get_error() {
        assert_error_snapshot(YarnBuildpackError::YarnCacheGet(create_cmd_error(
            "yarn config get cache-dir",
        )));
    }

    #[test]
    fn test_yarn_disable_global_cache_error() {
        assert_error_snapshot(YarnBuildpackError::YarnDisableGlobalCache(
            create_cmd_error("yarn config set enableGlobalCache false"),
        ));
    }

    #[test]
    fn test_yarn_install_error() {
        assert_error_snapshot(YarnBuildpackError::YarnInstall(create_cmd_error(
            "yarn install",
        )));
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
    fn test_yarn_version_resolve_error() {
        assert_error_snapshot(YarnBuildpackError::YarnVersionResolve(
            Requirement::parse("1.2.3").unwrap(),
        ));
    }

    #[test]
    fn test_yarn_default_parse_error() {
        assert_error_snapshot(YarnBuildpackError::YarnDefaultParse(create_version_error()));
    }

    #[test]
    fn test_yarn_node_build_scripts_metadata_error_for_invalid_enabled_value() {
        assert_error_snapshot(YarnBuildpackError::NodeBuildScriptsMetadata(
            NodeBuildScriptsMetadataError::InvalidEnabledValue(toml::value::Value::String(
                "test".to_string(),
            )),
        ));
    }

    #[test]
    fn test_yarn_node_build_scripts_metadata_error_for_invalid_skip_pruning_value() {
        assert_error_snapshot(YarnBuildpackError::NodeBuildScriptsMetadata(
            NodeBuildScriptsMetadataError::InvalidSkipPruningValue(toml::value::Value::String(
                "test".to_string(),
            )),
        ));
    }

    #[test]
    fn test_yarn_fetch_yarn_packument_error() {
        assert_error_snapshot(YarnBuildpackError::FetchYarnPackument(
            PackumentLayerError::ParsePackument(create_json_error()),
        ));
    }

    #[test]
    fn test_yarn_prune_dev_dependencies_error() {
        assert_error_snapshot(YarnBuildpackError::PruneYarnDevDependencies(
            create_cmd_error("yarn heroku prune"),
        ));
    }

    #[test]
    fn test_yarn_get_node_linker_parse_error() {
        assert_error_snapshot(YarnBuildpackError::YarnGetNodeLinker(
            GetNodeLinkerError::Parse(UnknownNodeLinker("test-linker".to_string())),
        ));
    }

    #[test]
    fn test_yarn_get_node_linker_command_error() {
        assert_error_snapshot(YarnBuildpackError::YarnGetNodeLinker(
            GetNodeLinkerError::Command(create_cmd_error("yarn config get nodeLinker")),
        ));
    }

    #[test]
    fn test_yarn_install_prune_plugin_error() {
        assert_error_snapshot(YarnBuildpackError::InstallPrunePluginError(
            create_io_error("Out of disk space"),
        ));
    }

    fn assert_error_snapshot(error: impl Into<Error<YarnBuildpackError>>) {
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
        Version::parse("not.a.version").unwrap_err()
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
