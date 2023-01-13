use indoc::formatdoc;
use libherokubuildpack::log::log_error;

use crate::CorepackBuildpackError;

pub(crate) fn on_error(err: libcnb::Error<CorepackBuildpackError>) {
    match err {
        libcnb::Error::BuildpackError(bp_err) => on_buildpack_error(bp_err),
        libcnb_err => log_error(
            "heroku/nodejs-corepack internal buildpack error",
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

fn on_buildpack_error(bp_err: CorepackBuildpackError) {
    match bp_err {
        CorepackBuildpackError::CorepackEnable(err) => on_corepack_cmd_error(
            "Unable to install corepack shims via `corepack enable`",
            err,
        ),
        CorepackBuildpackError::CorepackVersion(err) => on_corepack_cmd_error(
            "Unable to check corepack version via `corepack --version`",
            err,
        ),
        CorepackBuildpackError::CorepackPrepare(err) => on_corepack_cmd_error(
            "Unable to download package manager via `corepack prepare`",
            err,
        ),
        CorepackBuildpackError::ShimLayer(err) => on_layer_error("shim", &err),
        CorepackBuildpackError::ManagerLayer(err) => on_layer_error("manager", &err),
        CorepackBuildpackError::PackageJson(err) => log_error(
            "heroku/nodejs-corepack package.json error",
            formatdoc! {"
                There was an error while attempting to parse this project's
                package.json file. Please make sure it is present and properly
                formatted.

                Details: {err}
            "},
        ),
        CorepackBuildpackError::PackageManagerMissing => log_error(
            "heroku/nodejs-corepack packageManager error",
            formatdoc! {"
                There was an error decoding the `packageManager` key from
                this project's package.json. Please make sure it is present
                and properly formatted (for example: \"yarn@3.1.2\").
            "},
        ),
    };
}

fn on_corepack_cmd_error(err_context: &str, cmd_err: crate::cmd::Error) {
    let header = "heroku/nodejs-corepack corepack command error";
    match cmd_err {
        crate::cmd::Error::Exit(exit_err) => log_error(
            header,
            formatdoc! {"
                {err_context}. The command did not exit successfully.

                Details: {exit_err}
            "},
        ),
        crate::cmd::Error::Parse(output) => log_error(
            header,
            formatdoc! {"
                {err_context}. Error parsing the command output.

                Output: {output}
            "},
        ),
        crate::cmd::Error::Spawn(spawn_err) => log_error(
            header,
            formatdoc! {"
                {err_context}. Error spawning the command. Please ensure corepack
                was installed by another buildpack, such as heroku/nodejs-engine.

                Details: {spawn_err}
            "},
        ),
        crate::cmd::Error::Wait(wait_err) => log_error(
            header,
            formatdoc! {"
                {err_context}. Error waiting for the command to exit.

                Details: {wait_err}
            "},
        ),
    }
}

fn on_layer_error(layer_name: &str, io_err: &std::io::Error) {
    log_error(
        "heroku/nodejs-corepack layer creation error",
        formatdoc! {"
            Couldn't create the {layer_name} layer. An unexpected I/O error
            occurred.

            Details: {io_err}
        "},
    );
}
