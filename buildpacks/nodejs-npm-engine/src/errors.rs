use crate::layers::npm_engine::NpmEngineLayerError;
use crate::{node, npm};
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt;
use commons::output::fmt::DEBUG_INFO;
use heroku_nodejs_utils::package_json::PackageJsonError;
use heroku_nodejs_utils::vrs::Requirement;
use indoc::formatdoc;
use libcnb::Error;
use std::fmt::Display;
use std::io::stdout;

const USE_DEBUG_INFORMATION_AND_RETRY_BUILD: &str = "\
Use the debug information above to troubleshoot and retry your build.";

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack or framework, reproduce the issue \
locally with a minimal example. Open an issue in the buildpack's GitHub repository and include the details.";

#[derive(Debug)]
pub(crate) enum NpmEngineBuildpackError {
    PackageJson(PackageJsonError),
    MissingNpmEngineRequirement,
    InventoryParse(toml::de::Error),
    NpmVersionResolve(Requirement),
    NpmEngineLayer(NpmEngineLayerError),
    NodeVersion(node::VersionError),
    NpmVersion(npm::VersionError),
}

pub(crate) fn on_error(error: Error<NpmEngineBuildpackError>) {
    let logger = BuildLog::new(stdout()).without_buildpack_name();
    match error {
        Error::BuildpackError(buildpack_error) => on_buildpack_error(buildpack_error, logger),
        framework_error => on_framework_error(framework_error, logger),
    }
}

fn on_buildpack_error(error: NpmEngineBuildpackError, logger: Box<dyn StartedLogger>) {
    match error {
        NpmEngineBuildpackError::PackageJson(e) => on_package_json_error(e, logger),
        NpmEngineBuildpackError::MissingNpmEngineRequirement => {
            on_missing_npm_engine_requirement_error(logger)
        }
        NpmEngineBuildpackError::InventoryParse(e) => on_inventory_parse_error(e, logger),
        NpmEngineBuildpackError::NpmVersionResolve(requirement) => {
            on_npm_version_resolve_error(requirement, logger)
        }
        NpmEngineBuildpackError::NpmEngineLayer(e) => on_npm_engine_layer_error(e, logger),
        NpmEngineBuildpackError::NodeVersion(e) => on_node_version_error(e, logger),
        NpmEngineBuildpackError::NpmVersion(e) => on_npm_version_error(e, logger),
    }
}

fn on_package_json_error(error: PackageJsonError, logger: Box<dyn StartedLogger>) {
    match error {
        PackageJsonError::AccessError(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Error reading {package_json}.

                    The Node buildpack requires {package_json} to complete the build but the file can’t be read. 
                    
                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", package_json = fmt::value("package.json")});
        }
        PackageJsonError::ParseError(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Error reading {package_json}.

                    The Node buildpack requires {package_json} to complete the build but the file \
                    can’t be parsed. Check the formatting in your file. 
                    
                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", package_json = fmt::value("package.json")});
        }
    }
}

fn on_missing_npm_engine_requirement_error(logger: Box<dyn StartedLogger>) {
    logger.announce().error(&formatdoc! {"
        Missing {engines_key} key in {package_json}.
        
        This buildpack requires the `engines.npm` key to determine which engine versions to install.

        Retry your build. 

        {SUBMIT_AN_ISSUE}
    ", engines_key = fmt::value("engines.npm"), package_json = fmt::value("package.json") });
}

fn on_inventory_parse_error(error: toml::de::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            Failed to load available {npm} versions.

            An unexpected error occurred while loading the available {npm} versions.
        
            {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

            {SUBMIT_AN_ISSUE}
        ", npm = fmt::value("npm") })
}

fn on_npm_version_resolve_error(requirement: Requirement, logger: Box<dyn StartedLogger>) {
    logger.announce().error(&formatdoc! {"
            Error resolving requested {npm} version {requested_version}.
            
            Can’t find the `npm` version that matches the requested version declared in {package_json} ({requested_version}).
    
            Verify that the requested version range matches a published version of {npm} by checking \
            https://www.npmjs.com/package/npm?activeTab=versions or trying the following command:
            
                $ npm show 'npm@{requirement}' versions
    
            Update the {engines_key} field in {package_json} to a published {npm} version.
    
            {SUBMIT_AN_ISSUE}
        ",
        npm = fmt::value("npm"),
        requested_version = fmt::value(requirement.to_string()),
        package_json = fmt::value("package.json"),
        engines_key = fmt::value("engines.npm")
    })
}

fn on_npm_engine_layer_error(error: NpmEngineLayerError, logger: Box<dyn StartedLogger>) {
    match error {
        NpmEngineLayerError::Download(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Failed to download {npm}.

                    An unexpected error occurred while downloading the {npm} package. This error can occur due to an unstable network connection.

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = fmt::value("npm") });
        }
        NpmEngineLayerError::OpenTarball(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Can’t open the downloaded {npm} package.

                    An unexpected error occurred while opening the downloaded {npm} package file. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = fmt::value("npm") });
        }
        NpmEngineLayerError::DecompressTarball(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Can’t extract the downloaded {npm} package contents.

                    An unexpected error occurred while extracting the contents of the downloaded {npm} package file.

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = fmt::value("npm") });
        }
        NpmEngineLayerError::RemoveExistingNpmInstall(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Can’t remove the existing {npm} installation.

                    An unexpected error occurred while removing the existing {npm} installation. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = fmt::value("npm") });
        }
        NpmEngineLayerError::InstallNpm(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Can’t install the downloaded {npm} installation.
    
                    An unexpected error occurred while installing the downloaded {npm} package. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
            ", npm = fmt::value("npm") });
        }
    }
}

fn on_node_version_error(error: node::VersionError, logger: Box<dyn StartedLogger>) {
    match error {
        node::VersionError::Command(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Failed to determine {node} version information.
    
                    An unexpected error occurred while executing {node_version}. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", node = fmt::value("Node"), node_version = fmt::value(e.name()) });
        }
        node::VersionError::Parse(stdout) => {
            logger.announce().error(&formatdoc! {"
                Failed to parse {node} version information.
    
                An unexpected error occurred while parsing version information from {output}. 
                
                {SUBMIT_AN_ISSUE}
            ", node = fmt::value("Node"), output = fmt::value(stdout) });
        }
    }
}

fn on_npm_version_error(error: npm::VersionError, logger: Box<dyn StartedLogger>) {
    match error {
        npm::VersionError::Command(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Failed to determine {npm} version information.

                    An unexpected error occurred while executing {npm_version}.  
                    
                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}
        
                    {SUBMIT_AN_ISSUE}
                ", npm = fmt::value("npm"), npm_version = fmt::value(e.name())});
        }
        npm::VersionError::Parse(stdout) => {
            logger.announce().error(&formatdoc! {"
                Failed to parse {npm} version information.
    
                An unexpected error occurred while parsing version information from {output}. 
                
                {SUBMIT_AN_ISSUE}
            ", npm = fmt::value("npm"), output = fmt::value(stdout) });
        }
    }
}

fn on_framework_error(error: Error<NpmEngineBuildpackError>, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, &error)
        .announce()
        .error(&formatdoc! {"
            {buildpack_name} internal buildpack error.
    
            The framework used by this buildpack encountered an unexpected error.

            If you can't deploy to Heroku due to this issue, check the official Heroku Status page at \
            status.heroku.com for any ongoing incidents. After all incidents resolve, retry your build.

            If the issue persists and you think you found a bug in the buildpack or framework, reproduce \
            the issue locally with a minimal example. Open an issue in the buildpack's GitHub repository \
            and include the details.

        ", buildpack_name = fmt::value("heroku/nodejs-npm-engine") });
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
