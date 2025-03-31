use crate::corepack::cmd::CorepackVersionError;
use crate::CorepackBuildpackError;
use bullet_stream::state::Bullet;
use bullet_stream::Print;
use fun_run::CmdError;
use indoc::formatdoc;
use std::io::{stderr, Stderr};

pub(crate) fn on_error(err: CorepackBuildpackError) {
    let log = Print::new(stderr()).without_header();
    on_buildpack_error(err, log);
}

fn on_buildpack_error(bp_err: CorepackBuildpackError, log: Print<Bullet<Stderr>>) {
    match bp_err {
        CorepackBuildpackError::CorepackEnable(err) => on_corepack_cmd_error(
            "Unable to install corepack shims via `corepack enable`",
            &err,
            log,
        ),
        CorepackBuildpackError::CorepackPrepare(err) => on_corepack_cmd_error(
            "Unable to download package manager via `corepack prepare`",
            &err,
            log,
        ),
        CorepackBuildpackError::CorepackVersion(corepack_version_error) => {
            match corepack_version_error {
                CorepackVersionError::Parse(err) => {
                    log.error(formatdoc! { "
                        Unable to check corepack version via `corepack --version`

                        Corepack output couldn't be parsed: {err}
                    " });
                }
                CorepackVersionError::Command(err) => on_corepack_cmd_error(
                    "Unable to check corepack version via `corepack --version`",
                    &err,
                    log,
                ),
            }
        }
        CorepackBuildpackError::ShimLayer(err) => on_layer_error("shim", &err, log),
        CorepackBuildpackError::ManagerLayer(err) => on_layer_error("manager", &err, log),
        CorepackBuildpackError::PackageJson(err) => {
            log.error(formatdoc! {"
                heroku/nodejs-corepack package.json error
                
                There was an error while attempting to parse this project's \
                package.json file. Please make sure it is present and properly \
                formatted.
    
                Details: {err}
            "});
        }
        CorepackBuildpackError::PackageManagerMissing => {
            log.error(formatdoc! {"
                heroku/nodejs-corepack packageManager error

                There was an error decoding the `packageManager` key from \
                this project's package.json. Please make sure it is present \
                and properly formatted (for example: \"yarn@3.1.2\").
            "});
        }
    };
}

fn on_corepack_cmd_error(err_context: &str, cmd_err: &CmdError, log: Print<Bullet<Stderr>>) {
    log.error(formatdoc! { "
        heroku/nodejs-corepack corepack command error

        {err_context}. The command did not exit successfully.

        Details: {cmd_err}
    " });
}

fn on_layer_error(layer_name: &str, io_err: &std::io::Error, log: Print<Bullet<Stderr>>) {
    log.error(formatdoc! { "
        heroku/nodejs-corepack layer creation error

        Couldn't create the {layer_name} layer. An unexpected I/O error occurred.
    
        Details: {io_err}
    "});
}
