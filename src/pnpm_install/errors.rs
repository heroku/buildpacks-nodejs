use super::main::PnpmInstallBuildpackError;
use crate::utils::error_handling::{ErrorMessage, on_package_json_error};

#[allow(clippy::too_many_lines)]
pub(crate) fn on_pnpm_install_buildpack_error(error: PnpmInstallBuildpackError) -> ErrorMessage {
    match error {
        PnpmInstallBuildpackError::PackageJson(e) => on_package_json_error(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::package_json::PackageJsonError;
    use bullet_stream::strip_ansi;
    use insta::{assert_snapshot, with_settings};
    use std::io;
    use test_support::test_name;

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
}
