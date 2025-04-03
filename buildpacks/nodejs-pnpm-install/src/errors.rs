use crate::PnpmInstallBuildpackError;
use bullet_stream::state::Bullet;
use bullet_stream::{style, Print};
use heroku_nodejs_utils::buildplan::{
    NodeBuildScriptsMetadataError, NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME,
};
use indoc::formatdoc;
use std::io::{stderr, Stderr};

pub(crate) fn on_error(err: libcnb::Error<PnpmInstallBuildpackError>) {
    let log = Print::new(stderr()).without_header();
    match err {
        libcnb::Error::BuildpackError(bp_err) => on_buildpack_error(bp_err, log),
        libcnb_err => {
            log.error(formatdoc! {"
                heroku/nodejs-pnpm internal buildpack error
        
                An unexpected internal error was reported by the framework used \
                by this buildpack.
        
                If the issue persists, consider opening an issue on the GitHub \
                repository. If you are unable to deploy to Heroku as a result \
                of this issue, consider opening a ticket for additional support.
        
                Details: {libcnb_err}
            "});
        }
    }
}

fn on_buildpack_error(bp_err: PnpmInstallBuildpackError, log: Print<Bullet<Stderr>>) {
    match bp_err {
        PnpmInstallBuildpackError::BuildScript(err) => {
            log.error(formatdoc! {"
                heroku/nodejs-pnpm build script error

                There was an error while attempting to run a build script \
                from this project's package.json. 

                Details: {err}
            "});
        }
        PnpmInstallBuildpackError::PackageJson(err) => {
            log.error(formatdoc! {"
                heroku/nodejs-pnpm package.json error

                There was an error while attempting to parse this project's \
                package.json file. Please make sure it is present and properly \
                formatted.
    
                Details: {err}
            "});
        }
        PnpmInstallBuildpackError::PnpmInstall(err) => {
            log.error(formatdoc! {"
                pnpm install error

                There was an error while attempting to install dependencies \
                with pnpm. 
    
                Details: {err}
            "});
        }
        PnpmInstallBuildpackError::PnpmDir(err) => {
            log.error(formatdoc! {"
                directory error

                There was an error while attempting to configure a pnpm \
                store to a buildpack layer directory. 

                Details: {err}
            "});
        }
        PnpmInstallBuildpackError::PnpmStorePrune(err) => {
            log.error(formatdoc! {"
                store error

                There was an error while attempting to prune the pnpm \
                content-addressable store.

                Details: {err}
            "});
        }
        PnpmInstallBuildpackError::VirtualLayer(err) => {
            log.error(formatdoc! {"
                virtual store layer error

                There was an error while attempting to create the virtual \
                store layer for pnpm's installed dependencies.

                Details: {err}
            "});
        }
        PnpmInstallBuildpackError::NodeBuildScriptsMetadata(err) => {
            let NodeBuildScriptsMetadataError::InvalidEnabledValue(value) = err;
            let value_type = value.type_str();
            let requires_metadata = style::value("[requires.metadata]");
            let buildplan_name = style::value(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME);
            log.error(formatdoc! {"
                metadata error in {buildplan_name} build plan

                A participating buildpack has set invalid {requires_metadata} for the \
                build plan named {buildplan_name}.

                Expected metadata format:
                [requires.metadata]
                enabled = <bool>

                But was:
                [requires.metadata]
                enabled = <{value_type}>
            "});
        }
    }
}
