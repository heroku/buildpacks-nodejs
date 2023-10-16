use crate::npm;
use crate::BUILDPACK_NAME;
use commons::fun_run::CmdError;
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt;
use commons::output::fmt::DEBUG_INFO;
use heroku_nodejs_utils::application;
use heroku_nodejs_utils::package_json::PackageJsonError;
use indoc::formatdoc;
use std::fmt::Display;
use std::io::stdout;

const USE_DEBUG_INFORMATION_AND_RETRY_BUILD: &str = "\
Use the debug information above to troubleshoot and retry your build.";

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack or framework, reproduce the issue \
locally with a minimal example. Open an issue in the buildpack's GitHub repository and include the details.";

#[derive(Debug)]
pub(crate) enum NpmInstallBuildpackError {
    Application(application::Error),
    BuildScript(CmdError),
    Detect(std::io::Error),
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
        NpmInstallBuildpackError::Application(e) => on_application_error(e, logger),
        NpmInstallBuildpackError::BuildScript(e) => on_build_script_error(e, logger),
        NpmInstallBuildpackError::Detect(e) => on_detect_error(e, logger),
        NpmInstallBuildpackError::NpmInstall(e) => on_npm_install_error(e, logger),
        NpmInstallBuildpackError::NpmSetCacheDir(e) => on_set_cache_dir_error(e, logger),
        NpmInstallBuildpackError::NpmVersion(e) => on_npm_version_error(e, logger),
        NpmInstallBuildpackError::PackageJson(e) => on_package_json_error(e, logger),
    }
}

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
                ", package_json = fmt::value("package.json") });
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

fn on_set_cache_dir_error(error: CmdError, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Failed to set the {npm} cache directory.

            An unexpected error occurred while setting the {npm} cache directory. 
                    
            {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

            {SUBMIT_AN_ISSUE}
        ", npm = fmt::value("npm") });
}

fn on_npm_version_error(error: npm::VersionError, logger: Box<dyn StartedLogger>) {
    match error {
        npm::VersionError::Command(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Failed to determine {npm} version information.

                    An unexpected error occurred while executing {npm_version}.  
                    
                    {SUBMIT_AN_ISSUE}
                ", npm = fmt::value("npm"), npm_version = fmt::value(e.name()) });
        }
        npm::VersionError::Parse(stdout, e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Failed to parse {npm} version information.
        
                    An unexpected error occurred while parsing version information from {output}. 
                    
                    {SUBMIT_AN_ISSUE}
                ", npm = fmt::value("npm"), output = fmt::value(stdout) });
        }
    }
}

fn on_npm_install_error(error: CmdError, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Failed to install Node modules.

            The {buildpack_name} uses the command {npm_install} to install your Node modules. This command \
            failed and the buildpack cannot continue. See the log output above for more information.

            This error can occur due to an unstable network connection. Ensure that this command runs locally \
            without error and retry your build.
            
            If that doesn’t help, check the status of the upstream Node module repository service at https://status.npmjs.org/.
        ", npm_install = fmt::value(error.name()), buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_build_script_error(error: CmdError, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Failed to execute build script.

            The {buildpack_name} allows customization of the build process by executing the following scripts \
            if they are defined in {package_json}:
            - {heroku_prebuild} 
            - {heroku_build} or {build} 
            - {heroku_postbuild}

            An unexpected error occurred while executing {build_script}. See the log output above for more information.

            Ensure that this command runs locally without error and retry your build.
        ",
            build_script = fmt::value(error.name()),
            buildpack_name = fmt::value(BUILDPACK_NAME),
            package_json = fmt::value("package.json"),
            heroku_prebuild = fmt::value("heroku-prebuild"),
            heroku_build = fmt::value("heroku-build"),
            build = fmt::value("build"),
            heroku_postbuild = fmt::value("heroku-postbuild"),
        });
}

fn on_application_error(error: application::Error, logger: Box<dyn StartedLogger>) {
    logger.announce().error(&error.to_string());
}

fn on_detect_error(error: std::io::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Unable to complete buildpack detection.

            An unexpected error occurred while determining if the {buildpack_name} should be \
            run for this application. See the log output above for more information. 
        ", buildpack_name = fmt::value(BUILDPACK_NAME) });
}

fn on_framework_error(
    error: libcnb::Error<NpmInstallBuildpackError>,
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
