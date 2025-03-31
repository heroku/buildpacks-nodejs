use crate::common::buildplan::{NodeBuildScriptsMetadataError, NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME};
use crate::common::package_json::PackageJsonError;
use crate::npm_install::main::NpmInstallBuildpackError;
use crate::npm_install::npm;
use bullet_stream::state::Bullet;
use bullet_stream::{style, Print};
use fun_run::CmdError;
use indoc::formatdoc;
use std::fmt::Display;
use std::io;
use std::io::{stderr, Stderr};

const USE_DEBUG_INFORMATION_AND_RETRY_BUILD: &str = "\
Use the debug information above to troubleshoot and retry your build.";

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";

pub(crate) fn on_error(error: NpmInstallBuildpackError) {
    let logger = Print::new(stderr()).without_header();
    on_buildpack_error(error, logger);
}

fn on_buildpack_error(error: NpmInstallBuildpackError, logger: Print<Bullet<Stderr>>) {
    match error {
        NpmInstallBuildpackError::BuildScript(e) => on_build_script_error(&e, logger),
        NpmInstallBuildpackError::Detect(e) => on_detect_error(&e, logger),
        NpmInstallBuildpackError::NodeBuildScriptsMetadata(e) => {
            on_node_build_scripts_metadata_error(e, logger);
        }
        NpmInstallBuildpackError::NpmInstall(e) => on_npm_install_error(&e, logger),
        NpmInstallBuildpackError::NpmSetCacheDir(e) => on_set_cache_dir_error(&e, logger),
        NpmInstallBuildpackError::NpmVersion(e) => on_npm_version_error(e, logger),
        NpmInstallBuildpackError::PackageJson(e) => on_package_json_error(e, logger),
    }
}

fn on_node_build_scripts_metadata_error(
    error: NodeBuildScriptsMetadataError,
    logger: Print<Bullet<Stderr>>,
) {
    let NodeBuildScriptsMetadataError::InvalidEnabledValue(value) = error;
    let value_type = value.type_str();
    logger.error(formatdoc! { "
        A participating buildpack has set invalid `[requires.metadata]` for the build plan \
        named `{NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME}`.
        
        Expected metadata format:
        [requires.metadata]
        enabled = <bool>
        
        But was:
        [requires.metadata]
        enabled = <{value_type}>     
    "});
}

fn on_package_json_error(error: PackageJsonError, logger: Print<Bullet<Stderr>>) {
    match error {
        PackageJsonError::AccessError(e) => {
            print_error_details(logger, &e)
                .error(formatdoc! {"
                    Error reading {package_json}.

                    This buildpack requires {package_json} to complete the build but the file can’t be read. 
                    
                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", package_json = style::value("package.json") });
        }
        PackageJsonError::ParseError(e) => {
            print_error_details(logger, &e)
                .error(formatdoc! {"
                    Error reading {package_json}.

                    This buildpack requires {package_json} to complete the build but the file \
                    can’t be parsed. Ensure {npm_install} runs locally to check the formatting in your file.
                    
                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", package_json = style::value("package.json"), npm_install = style::value("npm install") });
        }
    }
}

fn on_set_cache_dir_error(error: &CmdError, logger: Print<Bullet<Stderr>>) {
    print_error_details(logger, &error).error(formatdoc! {"
            Failed to set the {npm} cache directory.

            An unexpected error occurred while setting the {npm} cache directory. 
                    
            {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

            {SUBMIT_AN_ISSUE}
        ", npm = style::value("npm") });
}

fn on_npm_version_error(error: npm::VersionError, logger: Print<Bullet<Stderr>>) {
    match error {
        npm::VersionError::Command(e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    Failed to determine {npm} version information.

                    An unexpected error occurred while executing {npm_version}.  
                    
                    {SUBMIT_AN_ISSUE}
                ", npm = style::value("npm"), npm_version = style::value(e.name()) });
        }
        npm::VersionError::Parse(stdout, e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    Failed to parse {npm} version information.
        
                    An unexpected error occurred while parsing version information from {output}. 
                    
                    {SUBMIT_AN_ISSUE}
                ", npm = style::value("npm"), output = style::value(stdout) });
        }
    }
}

fn on_npm_install_error(error: &CmdError, logger: Print<Bullet<Stderr>>) {
    print_error_details(logger, &error)
        .error(formatdoc! {"
            Failed to install Node modules.

            The {buildpack_name} uses the command {npm_install} to install your Node modules. This command \
            failed and the buildpack cannot continue. See the log output above for more information.

            This error can occur due to an unstable network connection. Ensure that this command runs locally \
            without error (exit status = 0) and retry your build.
            
            If that doesn’t help, check the status of the upstream Node module repository service at https://status.npmjs.org/.
        ", npm_install = style::value(error.name()), buildpack_name = style::value("Node.js Buildpack") });
}

fn on_build_script_error(error: &CmdError, logger: Print<Bullet<Stderr>>) {
    print_error_details(logger, &error)
        .error(formatdoc! {"
            Failed to execute build script.

            The {buildpack_name} allows customization of the build process by executing the following scripts \
            if they are defined in {package_json}:
            - {heroku_prebuild} 
            - {heroku_build} or {build} 
            - {heroku_postbuild}

            An unexpected error occurred while executing {build_script}. See the log output above for more information.

            Ensure that this command runs locally without error and retry your build.
        ",
            build_script = style::value(error.name()),
            buildpack_name = style::value("Node.js Buildpack"),
            package_json = style::value("package.json"),
            heroku_prebuild = style::value("heroku-prebuild"),
            heroku_build = style::value("heroku-build"),
            build = style::value("build"),
            heroku_postbuild = style::value("heroku-postbuild"),
        });
}

fn on_detect_error(error: &io::Error, logger: Print<Bullet<Stderr>>) {
    print_error_details(logger, &error).error(formatdoc! {"
            Unable to complete buildpack detection.

            An unexpected error occurred while determining if the {buildpack_name} should be \
            run for this application. See the log output above for more information. 
        ", buildpack_name = style::value("Node.js Buildpack") });
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
