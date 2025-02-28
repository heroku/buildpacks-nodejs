use crate::BUILDPACK_NAME;
use bullet_stream::state::Bullet;
use bullet_stream::{style, Print};
use indoc::formatdoc;
use std::fmt::Display;
use std::io::{stderr, Stderr};

#[derive(Debug, Copy, Clone)]
pub(crate) enum PnpmEngineBuildpackError {
    CorepackRequired,
}

pub(crate) fn on_error(error: libcnb::Error<PnpmEngineBuildpackError>) {
    let logger = Print::new(stderr()).without_header();
    match error {
        libcnb::Error::BuildpackError(buildpack_error) => {
            on_buildpack_error(buildpack_error, logger);
        }
        framework_error => on_framework_error(&framework_error, logger),
    }
}

fn on_buildpack_error(error: PnpmEngineBuildpackError, logger: Print<Bullet<Stderr>>) {
    match error {
        PnpmEngineBuildpackError::CorepackRequired => {
            print_error_details(logger, &"Corepack Requirement Error").error(formatdoc! {"
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
            corepack_enable = style::command("corepack enable"),
            corepack_use_pnpm = style::command("corepack use pnpm@*"),
            heroku_nodejs_corepack = style::command("heroku/nodejs-corepack"),
            package_manager = style::value("packageManager"),
            pnpm = style::value("pnpm"),
            pnpm_lockfile = style::value("pnpm-lock.yaml"),
            package_json = style::value("package.json")});
        }
    }
}

fn on_framework_error(
    error: &libcnb::Error<PnpmEngineBuildpackError>,
    logger: Print<Bullet<Stderr>>,
) {
    print_error_details(logger, &error)
        .error(formatdoc! {"
            {buildpack_name} internal error.

            The framework used by this buildpack encountered an unexpected error.

            If you can't deploy to Heroku due to this issue, check the official Heroku Status page at \
            status.heroku.com for any ongoing incidents. After all incidents resolve, retry your build.

            If the issue persists and you think you found a bug in the buildpack or framework, reproduce \
            the issue locally with a minimal example. Open an issue in the buildpack's GitHub repository \
            and include the details.

        ", buildpack_name = style::value(BUILDPACK_NAME) });
}

fn print_error_details(
    logger: Print<Bullet<Stderr>>,
    error: &impl Display,
) -> Print<Bullet<Stderr>> {
    logger
        .bullet(style::important("DEBUG INFO:"))
        .sub_bullet(error.to_string())
        .done()
}
