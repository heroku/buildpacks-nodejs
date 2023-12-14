use crate::BUILDPACK_NAME;
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt;
use commons::output::fmt::DEBUG_INFO;
use indoc::formatdoc;
use std::fmt::Display;
use std::io::stdout;

#[derive(Debug, Copy, Clone)]
pub(crate) enum PnpmEngineBuildpackError {
    CorepackRequired,
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
                    A pnpm lockfile ({pnpm_lockfile}) was detected, but the
                    version of {pnpm} to install could not be determined.

                    {pnpm} may be installed via the {heroku_nodejs_corepack}
                    buildpack. It requires the desired {pnpm} version to be set
                    via the {package_manager} key in {package_json}.

                    To set {package_manager} in {package_json} to the latest
                    {pnpm}, run:

                    {corepack_enable}
                    {corepack_use_pnpm}

                    Then commit the result, and try again.
                ",
                corepack_enable = fmt::command("corepack enable"),
                corepack_use_pnpm = fmt::command("corepack use pnpm@*"),
                heroku_nodejs_corepack = fmt::command("heroku/nodejs-corepack"),
                package_manager = fmt::value("packageManager"),
                pnpm = fmt::value("pnpm"),
                pnpm_lockfile = fmt::value("pnpm-lock.yaml"),
                package_json = fmt::value("package.json")});
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
