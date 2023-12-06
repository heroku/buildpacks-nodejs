use crate::BUILDPACK_NAME;
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt;
use commons::output::fmt::DEBUG_INFO;
use heroku_nodejs_utils::package_json::PackageJsonError;
use indoc::formatdoc;
use std::fmt::Display;
use std::io::stdout;

#[derive(Debug)]
pub(crate) enum PnpmEngineBuildpackError {
    CorepackRequired,
    PackageJson(PackageJsonError),
}

pub(crate) fn on_error(error: libcnb::Error<PnpmEngineBuildpackError>) {
    let logger = BuildLog::new(stdout()).without_buildpack_name();
    match error {
        libcnb::Error::BuildpackError(buildpack_error) => {
            on_buildpack_error(buildpack_error, logger);
        }
        framework_error => on_framework_error(&framework_error, logger),
    }
}

fn on_buildpack_error(error: PnpmEngineBuildpackError, logger: Box<dyn StartedLogger>) {
    match error {
        PnpmEngineBuildpackError::CorepackRequired => {
            print_error_details(logger, &"Corepack Requirement Error")
                .announce()
                .error(&formatdoc! {"
                    {pnpm} dependencies were detected, but the version of {pnpm}
                    to install could not be determined.

                    This buildpack requires the {pnpm} version to be set via
                    the {package_manager} key in {package_json}.

                    To set {package_manager} in {package_json} to the latest {pnpm}, run:

                    {corepack_enable}
                    {corepack_use_pnpm}

                    Then commit the result, and try again.
                ",
                corepack_enable = fmt::command("corepack enable"),
                corepack_use_pnpm = fmt::command("corepack use pnpm@*"),
                package_manager = fmt::value("packageManager"),
                pnpm = fmt::value("pnpm"),
                package_json = fmt::value("package.json")});
        }
        PnpmEngineBuildpackError::PackageJson(pjson_err) => {
            on_package_json_error(pjson_err, logger)
        }
    }
}

const USE_DEBUG_INFORMATION_AND_RETRY_BUILD: &str = "\
Use the debug information above to troubleshoot and retry your build.";

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";
fn on_package_json_error(error: PackageJsonError, logger: Box<dyn StartedLogger>) {
    match error {
        PackageJsonError::AccessError(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Error reading {package_json}.

                    This buildpack requires {package_json} to complete the build but the file can’t be read.

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", package_json = fmt::value("package.json")});
        }
        PackageJsonError::ParseError(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Error reading {package_json}.

                    This buildpack requires {package_json} to complete the build but the file \
                    can’t be parsed. Ensure {npm_install} runs locally to check the formatting in your file.

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", package_json = fmt::value("package.json"), npm_install = fmt::value("npm install") });
        }
    }
}
fn on_framework_error(
    error: &libcnb::Error<PnpmEngineBuildpackError>,
    logger: Box<dyn StartedLogger>,
) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            {buildpack_name} internal error.

            The framework used by this buildpack encountered an unexpected error.

            If you can't deploy to Heroku due to this issue, check the official Heroku Status page at \
            status.heroku.com for any ongoing incidents. After all incidents resolve, retry your build.

            If the issue persists and you think you found a bug in the buildpack or framework, reproduce \
            the issue locally with a minimal example. Open an issue in the buildpack's GitHub repository \
            and include the details.

        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn print_error_details(
    logger: Box<dyn StartedLogger>,
    error: &impl Display,
) -> Box<dyn StartedLogger> {
    logger
        .section(DEBUG_INFO)
        .step(&error.to_string())
        .end_section()
}
