use crate::npm;
use commons::fun_run::CmdError;
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt::DEBUG_INFO;
use heroku_nodejs_utils::application;
use heroku_nodejs_utils::package_json::PackageJsonError;
use indoc::formatdoc;
use std::fmt::Display;
use std::io::stdout;

const OPEN_A_SUPPORT_TICKET: &str =
    "open a support ticket and include the full log output of this build";

const TRY_BUILDING_AGAIN: &str = "try again and to see if the error resolves itself";

#[derive(Debug)]
pub(crate) enum NpmInstallBuildpackError {
    Application(application::Error),
    BuildScript(CmdError),
    NpmInstall(CmdError),
    NpmSetCacheDir(CmdError),
    NpmVersion(npm::VersionError),
    PackageJson(PackageJsonError),
}

pub(crate) fn on_error(error: libcnb::Error<NpmInstallBuildpackError>) {
    let logger = BuildLog::new(stdout()).without_buildpack_name();
    match error {
        libcnb::Error::BuildpackError(buildpack_error) => {
            on_buildpack_error(buildpack_error, logger)
        }
        framework_error => on_framework_error(framework_error, logger),
    }
}

pub(crate) fn on_buildpack_error(error: NpmInstallBuildpackError, logger: Box<dyn StartedLogger>) {
    match error {
        NpmInstallBuildpackError::PackageJson(e) => on_package_json_error(e, logger),
        NpmInstallBuildpackError::NpmSetCacheDir(e) => on_set_cache_dir_error(e, logger),
        NpmInstallBuildpackError::NpmVersion(e) => on_npm_version_error(e, logger),
        NpmInstallBuildpackError::NpmInstall(e) => on_npm_install_error(e, logger),
        NpmInstallBuildpackError::BuildScript(e) => on_build_script_error(e, logger),
        NpmInstallBuildpackError::Application(e) => on_application_error(e, logger),
    }
}

fn on_package_json_error(error: PackageJsonError, logger: Box<dyn StartedLogger>) {
    match error {
        PackageJsonError::AccessError(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                    Failed to read package.json

                    An unexpected error occurred while reading package.json. 
                    
                    Please {TRY_BUILDING_AGAIN}. 
                "});
        }
        PackageJsonError::ParseError(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                    Failed to parse package.json

                    An unexpected error occurred while parsing package.json. 
                    
                    Please {TRY_BUILDING_AGAIN}.   
                "});
        }
    }
}

fn on_set_cache_dir_error(error: CmdError, logger: Box<dyn StartedLogger>) {
    let command = error.name().to_string();
    print_error_details(logger, error)
        .announce()
        .error(&formatdoc! {"
            Failed to set the npm cache directory

            An unexpected error occurred while executing `{command}`. 
            
            Please {TRY_BUILDING_AGAIN}. 
        "});
}

fn on_npm_version_error(error: npm::VersionError, logger: Box<dyn StartedLogger>) {
    match error {
        npm::VersionError::Command(e) => {
            let command = e.name().to_string();
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                    Failed to determine npm version information
        
                    An unexpected error occurred while executing `{command}`. 
                    
                    Please {TRY_BUILDING_AGAIN}. 
                "});
        }
        npm::VersionError::Parse(e) => {
            logger.announce().error(&formatdoc! {"
                    Failed to parse npm version information
        
                    An unexpected error occurred while parsing version information from `{e}`. 
                    
                    Please {TRY_BUILDING_AGAIN}. 
                "});
        }
    }
}

fn on_npm_install_error(error: CmdError, logger: Box<dyn StartedLogger>) {
    let command = error.name().to_string();
    print_error_details(logger, error)
        .announce()
        .error(&formatdoc! {"
            Failed to install node modules

            An unexpected error occurred while executing `{command}`. See the log output above for more information.

            In some cases, this happens due to an unstable network connection. Please {TRY_BUILDING_AGAIN}.
            
            If that does not help, check the status of npm (the upstream Node module repository service) here:
            https://status.npmjs.org/
        "});
}

fn on_build_script_error(error: CmdError, logger: Box<dyn StartedLogger>) {
    let command = error.name().to_string();
    print_error_details(logger, error)
        .announce()
        .error(&formatdoc! {"
            Failed to execute build script

            An unexpected error occurred while executing `{command}`. 
            
            Please try running this command locally to verify that it works as expected. 
        "});
}

fn on_application_error(error: application::Error, logger: Box<dyn StartedLogger>) {
    logger.announce().error(&error.to_string());
}

fn on_framework_error(
    error: libcnb::Error<NpmInstallBuildpackError>,
    logger: Box<dyn StartedLogger>,
) {
    print_error_details(logger, error)
        .announce()
        .error(&formatdoc! {"
            Internal buildpack error

            An unexpected internal error was reported by the framework used by this buildpack. 
            
            Please {OPEN_A_SUPPORT_TICKET}.
        "});
}

fn print_error_details(
    logger: Box<dyn StartedLogger>,
    error: impl Display,
) -> Box<dyn StartedLogger> {
    logger
        .section(DEBUG_INFO)
        .step(&error.to_string())
        .end_section()
}
