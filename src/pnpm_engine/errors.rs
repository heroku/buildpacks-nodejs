use crate::pnpm_engine::main::PnpmEngineBuildpackError;
use bullet_stream::state::Bullet;
use bullet_stream::{style, Print};
use indoc::formatdoc;
use std::fmt::Display;
use std::io::{stderr, Stderr};

pub(crate) fn on_error(error: PnpmEngineBuildpackError) {
    let logger = Print::new(stderr()).without_header();
    on_buildpack_error(error, logger);
}

fn on_buildpack_error(error: PnpmEngineBuildpackError, logger: Print<Bullet<Stderr>>) {
    match error {
        PnpmEngineBuildpackError::CorepackRequired => {
            print_error_details(logger, &"Corepack Requirement Error").error(formatdoc! {"
                    A pnpm lockfile ({pnpm_lockfile}) was detected, but the
                    version of {pnpm} to install could not be determined.

                    {pnpm} may be installed via the {heroku_nodejs}
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
            heroku_nodejs = style::command("heroku/nodejs"),
            package_manager = style::value("packageManager"),
            pnpm = style::value("pnpm"),
            pnpm_lockfile = style::value("pnpm-lock.yaml"),
            package_json = style::value("package.json")});
        }
    }
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
