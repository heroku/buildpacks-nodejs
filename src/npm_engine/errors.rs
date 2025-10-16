use super::install_npm::NpmInstallError;
use super::main::NpmEngineBuildpackError;
use super::{node, npm};
use crate::utils::error_handling::ErrorType::{Internal, UserFacing};
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
    on_package_json_error,
};
use crate::utils::npm_registry::PackumentLayerError;
use crate::utils::vrs::Requirement;
use bullet_stream::style;
use indoc::formatdoc;

pub(crate) fn on_npm_engine_error(error: NpmEngineBuildpackError) -> ErrorMessage {
    match error {
        NpmEngineBuildpackError::PackageJson(e) => on_package_json_error(e),
        NpmEngineBuildpackError::MissingNpmEngineRequirement => {
            on_missing_npm_engine_requirement_error()
        }
        NpmEngineBuildpackError::NpmVersionResolve(requirement) => {
            on_npm_version_resolve_error(&requirement)
        }
        NpmEngineBuildpackError::NpmInstall(e) => on_npm_install_error(e),
        NpmEngineBuildpackError::NodeVersion(e) => on_node_version_error(e),
        NpmEngineBuildpackError::NpmVersion(e) => on_npm_version_error(e),
        NpmEngineBuildpackError::FetchNpmPackument(e) => on_fetch_npm_packument_error(&e),
    }
}

fn on_missing_npm_engine_requirement_error() -> ErrorMessage {
    let engines_key = style::value("engines.npm");
    let package_json = style::value("package.json");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::No))
        .header(format!("Missing {engines_key} key in {package_json}"))
        .body(formatdoc! { "
            This buildpack requires the `engines.npm` key to determine which engine versions to \
            install.
        " })
        .create()
}

fn on_npm_version_resolve_error(requirement: &Requirement) -> ErrorMessage {
    let npm = style::value("npm");
    let requested_version = style::value(requirement.to_string());
    let npm_releases_url = style::url("https://www.npmjs.com/package/npm?activeTab=versions");
    let inventory_url = style::url(
        "https://github.com/heroku/buildpacks-nodejs/blob/main/buildpacks/nodejs-npm-engine/inventory.toml",
    );
    let npm_show_command = style::value(format!("npm show 'npm@{requirement}' versions"));
    let package_json = style::value("package.json");
    let engines_key = style::value("engines.npm");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!("Error resolving requested {npm} version {requested_version}"))
        .body(formatdoc! { "
            The requested npm version could not be resolved to a known release in this buildpack's \
            inventory of npm releases.
            
            Suggestions:
            - Confirm if this is a valid npm release at {npm_releases_url} or by running {npm_show_command}
            - Check if this buildpack includes the requested npm version in its inventory file at {inventory_url}
            - Update the {engines_key} field in {package_json} to a single version or version range that \
            includes a published {npm} version.
        " })
        .create()
}

fn on_npm_install_error(error: NpmInstallError) -> ErrorMessage {
    let npm = style::value("npm");
    let npm_status_url = style::url("https://status.npmjs.org/");
    match error {
        NpmInstallError::Download(e) =>
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header(format!("Failed to download {npm}"))
                .body(formatdoc! {"
                    An unexpected error occurred while downloading the {npm} package. This error can \
                    occur due to an unstable network connection or an issue with the npm repository.

                    Suggestions:
                    - Check the npm status page for any ongoing incidents ({npm_status_url})
                " })
                .debug_info(e.to_string())
                .create(),

        NpmInstallError::OpenTarball(path, e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
            .header(format!("Failed to open the downloaded {npm} package file"))
            .body(formatdoc! {"
                An unexpected I/O occurred while opening the downloaded {npm} package file at {path}.
            ", path = file_value(path) })
            .debug_info(e.to_string())
            .create(),

        NpmInstallError::DecompressTarball(path, e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
            .header(format!("Failed to extract {npm} package file"))
            .body(formatdoc! {"
                An unexpected I/O occurred while extracting the contents of the downloaded {npm} package file at {path}.
            ", path = file_value(path) })
            .debug_info(e.to_string())
            .create(),

        NpmInstallError::RemoveExistingNpmInstall(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
            .header(format!("Failed to remove the existing {npm} installation"))
            .body(formatdoc! {"
                An unexpected error occurred while removing the existing {npm} installation.
            " })
            .debug_info(e.to_string())
            .create(),

        NpmInstallError::InstallNpm(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
            .header(format!("Failed to install the downloaded {npm} package"))
            .body(formatdoc! {"
                An unexpected error occurred while installing the downloaded {npm} package.
            " })
            .debug_info(e.to_string())
            .create()
    }
}

fn on_node_version_error(error: node::VersionError) -> ErrorMessage {
    match error {
        node::VersionError::Command(e) => error_message()
            .error_type(Internal)
            .header("Failed to determine Node.js version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current Node.js version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        node::VersionError::Parse(stdout, e) => error_message()
            .error_type(Internal)
            .header("Failed to parse Node.js version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing Node.js version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

fn on_npm_version_error(error: npm::VersionError) -> ErrorMessage {
    match error {
        npm::VersionError::Command(e) => error_message()
            .error_type(Internal)
            .header("Failed to determine npm version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current npm version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        npm::VersionError::Parse(stdout, e) => error_message()
            .error_type(Internal)
            .header("Failed to parse npm version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing npm version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

fn on_fetch_npm_packument_error(error: &PackumentLayerError) -> ErrorMessage {
    let npm = style::value("npm");
    let npm_status_url = style::url("https://status.npmjs.org/");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header(format!("Failed to load available {npm} versions"))
        .body(formatdoc! { "
            An unexpected error occurred while loading the available {npm} versions. This error can \
            occur due to an unstable network connection or an issue with the npm registry.

            Suggestions:
            - Check the npm status page for any ongoing incidents ({npm_status_url})
        "})
        .debug_info(error.to_string())
        .create()
}

#[cfg(test)]
mod tests {
    use super::NpmEngineBuildpackError;
    use super::*;
    use crate::utils::package_json::PackageJsonError;
    use crate::utils::vrs::Version;
    use bullet_stream::strip_ansi;
    use fun_run::{CmdError, CommandWithName};
    use insta::{assert_snapshot, with_settings};
    use std::process::Command;
    use test_support::test_name;

    #[test]
    fn test_npm_engine_package_json_access_error() {
        assert_error_snapshot(NpmEngineBuildpackError::PackageJson(
            PackageJsonError::AccessError(create_io_error("test I/O error blah")),
        ));
    }

    #[test]
    fn test_npm_engine_package_json_parse_error() {
        assert_error_snapshot(NpmEngineBuildpackError::PackageJson(
            PackageJsonError::ParseError(create_json_error()),
        ));
    }

    #[test]
    fn test_npm_engine_missing_npm_engine_requirement_error() {
        assert_error_snapshot(NpmEngineBuildpackError::MissingNpmEngineRequirement);
    }

    #[test]
    fn test_npm_engine_npm_version_resolve_error() {
        assert_error_snapshot(NpmEngineBuildpackError::NpmVersionResolve(
            Requirement::parse("1.2.3").unwrap(),
        ));
    }

    #[test]
    fn test_npm_engine_npm_install_download_error() {
        assert_error_snapshot(NpmInstallError::Download(
            crate::utils::http::Error::Request("https://test/error".into(), create_reqwest_error()),
        ));
    }

    #[test]
    fn test_npm_engine_npm_install_open_tarball_error() {
        assert_error_snapshot(NpmInstallError::OpenTarball(
            "/layers/npm/install/npm.tgz".into(),
            create_io_error("Invalid permissions"),
        ));
    }

    #[test]
    fn test_npm_engine_npm_install_decompress_tarball_error() {
        assert_error_snapshot(NpmInstallError::DecompressTarball(
            "/layers/npm/install/npm.tgz".into(),
            create_io_error("Out of disk space"),
        ));
    }

    #[test]
    fn test_npm_engine_npm_install_remove_existing_npm_install_error() {
        assert_error_snapshot(NpmInstallError::RemoveExistingNpmInstall(create_cmd_error(
            "npm -g uninstall npm",
        )));
    }

    #[test]
    fn test_npm_engine_npm_install_install_npm_error() {
        assert_error_snapshot(NpmInstallError::InstallNpm(create_cmd_error(
            "npm -g install /layers/npm/install/package",
        )));
    }

    #[test]
    fn test_npm_engine_node_version_command_error() {
        assert_error_snapshot(NpmEngineBuildpackError::NodeVersion(
            node::VersionError::Command(create_cmd_error("node --version")),
        ));
    }

    #[test]
    fn test_npm_engine_node_version_parse_error() {
        assert_error_snapshot(NpmEngineBuildpackError::NodeVersion(
            node::VersionError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    #[test]
    fn test_npm_engine_npm_version_command_error() {
        assert_error_snapshot(NpmEngineBuildpackError::NpmVersion(
            npm::VersionError::Command(create_cmd_error("npm --version")),
        ));
    }

    #[test]
    fn test_npm_engine_npm_version_parse_error() {
        assert_error_snapshot(NpmEngineBuildpackError::NpmVersion(
            npm::VersionError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    #[test]
    fn test_npm_engine_fetch_npm_packument_error() {
        assert_error_snapshot(NpmEngineBuildpackError::FetchNpmPackument(
            PackumentLayerError::ReadPackument(create_io_error("Insufficient permissions")),
        ));
    }

    fn assert_error_snapshot(error: impl Into<NpmEngineBuildpackError>) {
        let error_message = strip_ansi(on_npm_engine_error(error.into()).to_string());
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
