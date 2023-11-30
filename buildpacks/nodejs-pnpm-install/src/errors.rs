use crate::cmd;
use crate::PnpmInstallBuildpackError;
use indoc::formatdoc;
use libherokubuildpack::log::log_error;

pub(crate) fn on_error(err: libcnb::Error<PnpmInstallBuildpackError>) {
    match err {
        libcnb::Error::BuildpackError(bp_err) => on_buildpack_error(bp_err),
        libcnb_err => log_error(
            "heroku/nodejs-pnpm internal buildpack error",
            formatdoc! {"
                An unexpected internal error was reported by the framework used
                by this buildpack.

                If the issue persists, consider opening an issue on the GitHub
                repository. If you are unable to deploy to Heroku as a result
                of this issue, consider opening a ticket for additional support.

                Details: {libcnb_err}
            "},
        ),
    };
}

fn on_buildpack_error(bp_err: PnpmInstallBuildpackError) {
    match bp_err {
        PnpmInstallBuildpackError::BuildScript(err) => {
            let (context, details) = get_cmd_error_context(err);
            log_error(
                "heroku/nodejs-pnpm build script error",
                formatdoc! {"
                    There was an error while attempting to run a build script
                    from this project's package.json. {context}

                    Details: {details}
                "},
            );
        }
        PnpmInstallBuildpackError::Symlink(err) => {
            log_error(
                "heroku/nodejs-pnpm-install symlink error",
                formatdoc! {"
                    There was an error while attempting to symlink the virtual
                    store to the application's `node_modules`.

                    Details: {err:?}
                "},
            );
        }
        PnpmInstallBuildpackError::PackageJson(err) => log_error(
            "heroku/nodejs-pnpm package.json error",
            formatdoc! {"
                There was an error while attempting to parse this project's
                package.json file. Please make sure it is present and properly
                formatted.

                Details: {err:?}
            "},
        ),
        PnpmInstallBuildpackError::PnpmInstall(err) => {
            let (context, details) = get_cmd_error_context(err);
            log_error(
                "heroku/nodejs-pnpm pnpm install error",
                formatdoc! {"
                    There was an error while attempting to install dependencies
                    with pnpm. {context}

                    Details: {details}
                "},
            );
        }
        PnpmInstallBuildpackError::PnpmDir(err) => {
            let (context, details) = get_cmd_error_context(err);
            log_error(
                "heroku/nodejs-pnpm directory error",
                formatdoc! {"
                    There was an error while attempting to configure a pnpm
                    store to a buildpack layer directory. {context}

                    Details: {details}
                "},
            );
        }
        PnpmInstallBuildpackError::PnpmStorePrune(err) => {
            let (context, details) = get_cmd_error_context(err);
            log_error(
                "heroku/nodejs-pnpm store error",
                formatdoc! {"
                    There was an error while attempting to prune the pnpm
                    content-addressable store. {context}

                    Details: {details}
                "},
            );
        }
    };
}

fn get_cmd_error_context(err: cmd::Error) -> (&'static str, String) {
    match err {
        cmd::Error::Spawn(io_err) => ("The operating system was unable to start the command.", format!("{io_err}")),
        cmd::Error::Wait(io_err) => ("The operating system was unable to wait for the command to finish. It was no longer running.", format!("{io_err}")),
        cmd::Error::Exit(exit_code) => ("The command exited with a non-zero exit code.", format!("Exit code {exit_code}"))
    }
}
