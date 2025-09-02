use crate::PnpmEngineBuildpackError;
use bullet_stream::style;
use heroku_nodejs_utils::error_handling::error_message_builder::SetIssuesUrl;
use heroku_nodejs_utils::error_handling::{
    on_framework_error, ErrorMessage, ErrorMessageBuilder, ErrorType, SuggestRetryBuild,
    SuggestSubmitIssue,
};
use indoc::formatdoc;

const BUILDPACK_NAME: &str = "Heroku Node.js pnpm Engine";

const ISSUES_URL: &str = "https://github.com/heroku/buildpacks-nodejs/issues";

pub(crate) fn on_error(error: libcnb::Error<PnpmEngineBuildpackError>) -> ErrorMessage {
    match error {
        libcnb::Error::BuildpackError(e) => on_buildpack_error(e),
        e => on_framework_error(BUILDPACK_NAME, ISSUES_URL, &e),
    }
}

// Wraps the error_message() builder to preset the issues_url field
fn error_message() -> ErrorMessageBuilder<SetIssuesUrl> {
    heroku_nodejs_utils::error_handling::error_message().issues_url(ISSUES_URL.to_string())
}

fn on_buildpack_error(error: PnpmEngineBuildpackError) -> ErrorMessage {
    match error {
        PnpmEngineBuildpackError::CorepackRequired => {
            let corepack_enable = style::command("corepack enable");
            let corepack_use_pnpm = style::command("corepack use pnpm@*");
            let heroku_nodejs_corepack = style::command("heroku/nodejs-corepack");
            let package_manager = style::value("packageManager");
            let pnpm = style::value("pnpm");
            let pnpm_lockfile = style::value("pnpm-lock.yaml");
            let package_json = style::value("package.json");

            error_message()
                .error_type(ErrorType::UserFacing(
                    SuggestRetryBuild::No,
                    SuggestSubmitIssue::No,
                ))
                .header("Corepack Requirement Error")
                .body(formatdoc! {"
                    A pnpm lockfile ({pnpm_lockfile}) was detected, but the \
                    version of {pnpm} to install could not be determined.

                    {pnpm} may be installed via the {heroku_nodejs_corepack} \
                    buildpack. It requires the desired {pnpm} version to be set \
                    via the {package_manager} key in {package_json}.

                    To set {package_manager} in {package_json} to the latest \
                    {pnpm}, run:

                    {corepack_enable}
                    {corepack_use_pnpm}

                    Then commit the result, and try again.
                " })
                .create()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use insta::{assert_snapshot, with_settings};
    use libcnb::Error;
    use test_support::test_name;

    #[test]
    fn test_pnpm_engine_corepack_required_error() {
        assert_error_snapshot(PnpmEngineBuildpackError::CorepackRequired);
    }

    fn assert_error_snapshot(error: impl Into<Error<PnpmEngineBuildpackError>>) {
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
}
