use super::install_npm::NpmInstallError;
use super::main::NpmEngineBuildpackError;
use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use bullet_stream::style;
use indoc::formatdoc;

pub(crate) fn on_npm_engine_error(error: NpmEngineBuildpackError) -> ErrorMessage {
    match error {
        NpmEngineBuildpackError::NpmInstall(e) => on_npm_install_error(e),
    }
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

#[cfg(test)]
mod tests {
    use super::NpmEngineBuildpackError;
    use super::*;
    use bullet_stream::strip_ansi;
    use fun_run::{CmdError, CommandWithName};
    use insta::{assert_snapshot, with_settings};
    use std::process::Command;
    use test_support::test_name;

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
