use crate::layers::npm_engine::NpmEngineLayerError;
use crate::{node, npm};
use commons::output::build_log::{BuildLog, Logger, StartedLogger};
use commons::output::fmt::DEBUG_INFO;
use heroku_nodejs_utils::package_json::PackageJsonError;
use heroku_nodejs_utils::vrs::Requirement;
use indoc::formatdoc;
use libcnb::Error;
use std::fmt::Display;
use std::io::stdout;

const OPEN_A_SUPPORT_TICKET: &str =
    "open a support ticket and include the full log output of this build";

const TRY_BUILDING_AGAIN: &str = "try again and to see if the error resolves itself";

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

fn on_missing_npm_engine_requirement_error(logger: Box<dyn StartedLogger>) {
    logger.announce().error(&formatdoc! {"
        Missing `engines.npm` key in package.json
        
        This buildpack should only run when `engines.npm` is present in package.json. If you do not
        have this field declared then please {OPEN_A_SUPPORT_TICKET}.
    "});
}

fn on_inventory_parse_error(error: toml::de::Error, logger: Box<dyn StartedLogger>) {
    print_error_details(logger, error)
        .announce()
        .error(&formatdoc! {"
        Failed to load npm inventory

        An unexpected error occurred while loading the available npm versions.
        
        Please {TRY_BUILDING_AGAIN}.
    "})
}

fn on_npm_version_resolve_error(requirement: Requirement, logger: Box<dyn StartedLogger>) {
    logger.announce().error(&formatdoc! {"
        Could not find a suitable version of npm that matches the requested version declared in 
        package.json ({requirement}). 
        
        Please verify that the requested version range matches a published version of 
        npm by checking https://www.npmjs.com/package/npm?activeTab=versions or trying the 
        following command:
        
           npm show 'npm@{requirement}' versions

        If no matching version exists then please update the `engines.npm` field in package.json to
        a different version range that does have a match.
        
        If a matching version exists then please {OPEN_A_SUPPORT_TICKET}.  
    "})
}

fn on_npm_engine_layer_error(error: NpmEngineLayerError, logger: Box<dyn StartedLogger>) {
    match error {
        NpmEngineLayerError::Download(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                Failed to download npm

                An unexpected error occurred while downloading the npm package. In some cases, this 
                happens due to an unstable network connection.

                Please {TRY_BUILDING_AGAIN}. 
            "});
        }
        NpmEngineLayerError::OpenTarball(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                Could not open the downloaded npm package file

                An unexpected error occurred while opening the downloaded npm package file.
                
                Please {TRY_BUILDING_AGAIN}. 
            "});
        }
        NpmEngineLayerError::DecompressTarball(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                Could not extract the downloaded npm package contents

                An unexpected error occurred while extracting the contents of the downloaded npm 
                package file. 
                
                Please {TRY_BUILDING_AGAIN}. 
            "});
        }
        NpmEngineLayerError::RemoveExistingNpmInstall(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                Unable to remove the existing npm installation

                An unexpected error occurred while removing the existing npm installation.
                
                Please {TRY_BUILDING_AGAIN}. 
            "});
        }
        NpmEngineLayerError::InstallNpm(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                Unable to install the downloaded npm package

                An unexpected error occurred while installing the downloaded npm package.
                
                Please {TRY_BUILDING_AGAIN}. 
            "});
        }
    }
}

fn on_node_version_error(error: node::VersionError, logger: Box<dyn StartedLogger>) {
    match error {
        node::VersionError::Command(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                    Failed to determine Node version information
    
                    An unexpected error occurred while executing `node --version`. 
                    
                    Please {TRY_BUILDING_AGAIN}. 
                "});
        }
        node::VersionError::Parse(e) => {
            logger.announce().error(&formatdoc! {"
                Failed to parse Node version information

                An unexpected error occurred while parsing version information from `{e}`. 
                
                Please {TRY_BUILDING_AGAIN}. 
            "});
        }
    }
}

fn on_npm_version_error(error: npm::VersionError, logger: Box<dyn StartedLogger>) {
    match error {
        npm::VersionError::Command(e) => {
            print_error_details(logger, e)
                .announce()
                .error(&formatdoc! {"
                    Failed to determine npm version information
    
                    An unexpected error occurred while executing `node --version`. 
                    
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

fn on_framework_error(error: Error<NpmEngineBuildpackError>, logger: Box<dyn StartedLogger>) {
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
