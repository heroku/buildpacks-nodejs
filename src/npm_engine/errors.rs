use crate::common::package_json::PackageJsonError;
use crate::common::vrs::Requirement;
use crate::npm_engine::install_npm::NpmEngineLayerError;
use crate::npm_engine::main::NpmEngineBuildpackError;
use crate::npm_engine::{node, npm};
use bullet_stream::state::Bullet;
use bullet_stream::{style, Print};
use indoc::formatdoc;
use std::fmt::Display;
use std::io::{stderr, Stderr};

const USE_DEBUG_INFORMATION_AND_RETRY_BUILD: &str = "\
Use the debug information above to troubleshoot and retry your build.";

const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";

pub(crate) fn on_error(error: NpmEngineBuildpackError) {
    let logger = Print::new(stderr()).without_header();
    on_buildpack_error(error, logger);
}

fn on_buildpack_error(error: NpmEngineBuildpackError, logger: Print<Bullet<Stderr>>) {
    match error {
        NpmEngineBuildpackError::PackageJson(e) => on_package_json_error(e, logger),
        NpmEngineBuildpackError::MissingNpmEngineRequirement => {
            on_missing_npm_engine_requirement_error(logger);
        }
        NpmEngineBuildpackError::InventoryParse(e) => on_inventory_parse_error(&e, logger),
        NpmEngineBuildpackError::NpmVersionResolve(requirement) => {
            on_npm_version_resolve_error(&requirement, logger);
        }
        NpmEngineBuildpackError::NpmEngineLayer(e) => on_npm_engine_layer_error(e, logger),
        NpmEngineBuildpackError::NodeVersion(e) => on_node_version_error(e, logger),
        NpmEngineBuildpackError::NpmVersion(e) => on_npm_version_error(e, logger),
    }
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
                ", package_json = style::value("package.json")});
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

fn on_missing_npm_engine_requirement_error(logger: Print<Bullet<Stderr>>) {
    logger.error(formatdoc! {"
        Missing {engines_key} key in {package_json}.
        
        This buildpack requires the `engines.npm` key to determine which engine versions to install.

        Retry your build. 

        {SUBMIT_AN_ISSUE}
    ", engines_key = style::value("engines.npm"), package_json = style::value("package.json") });
}

fn on_inventory_parse_error(error: &toml::de::Error, logger: Print<Bullet<Stderr>>) {
    print_error_details(logger, &error).error(formatdoc! {"
            Failed to load available {npm} versions.

            An unexpected error occurred while loading the available {npm} versions.
        
            {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

            {SUBMIT_AN_ISSUE}
        ", npm = style::value("npm") });
}

fn on_npm_version_resolve_error(requirement: &Requirement, logger: Print<Bullet<Stderr>>) {
    logger.error(formatdoc! {"
            Error resolving requested {npm} version {requested_version}.
            
            Can’t find the `npm` version that matches the requested version declared in {package_json} ({requested_version}).
    
            Verify that the requested version range matches a published version of {npm} by checking \
            https://www.npmjs.com/package/npm?activeTab=versions or trying the following command:
            
                $ npm show 'npm@{requirement}' versions
    
            Update the {engines_key} field in {package_json} to a single version or version range that \
            includes a published {npm} version.
    
            {SUBMIT_AN_ISSUE}
        ",
        npm = style::value("npm"),
        requested_version = style::value(requirement.to_string()),
        package_json = style::value("package.json"),
        engines_key = style::value("engines.npm")
    });
}

fn on_npm_engine_layer_error(error: NpmEngineLayerError, logger: Print<Bullet<Stderr>>) {
    match error {
        NpmEngineLayerError::Download(e) => {
            print_error_details(logger, &e)
                .error(formatdoc! {"
                    Failed to download {npm}.

                    An unexpected error occurred while downloading the {npm} package. This error can occur due to an unstable network connection.

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = style::value("npm") });
        }
        NpmEngineLayerError::OpenTarball(e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    An unexpected error occurred while opening the downloaded {npm} package file. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = style::value("npm") });
        }
        NpmEngineLayerError::DecompressTarball(e) => {
            print_error_details(logger, &e)
                .error(formatdoc! {"
                    An unexpected error occurred while extracting the contents of the downloaded {npm} package file.

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = style::value("npm") });
        }
        NpmEngineLayerError::RemoveExistingNpmInstall(e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    An unexpected error occurred while removing the existing {npm} installation. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", npm = style::value("npm") });
        }
        NpmEngineLayerError::InstallNpm(e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    An unexpected error occurred while installing the downloaded {npm} package. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
            ", npm = style::value("npm") });
        }
    }
}

fn on_node_version_error(error: node::VersionError, logger: Print<Bullet<Stderr>>) {
    match error {
        node::VersionError::Command(e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    Failed to determine {node} version information.
    
                    An unexpected error occurred while executing {node_version}. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", node = style::value("Node"), node_version = style::value(e.name()) });
        }
        node::VersionError::Parse(stdout, e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    Failed to parse {node} version information.
        
                    An unexpected error occurred while parsing version information from {output}. 
                    
                    {SUBMIT_AN_ISSUE}
                ", node = style::value("Node"), output = style::value(stdout) });
        }
    }
}

fn on_npm_version_error(error: npm::VersionError, logger: Print<Bullet<Stderr>>) {
    match error {
        npm::VersionError::Command(e) => {
            print_error_details(logger, &e).error(formatdoc! {"
                    Failed to determine {npm} version information.

                    An unexpected error occurred while executing {npm_version}.  
                    
                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}
        
                    {SUBMIT_AN_ISSUE}
                ", npm = style::value("npm"), npm_version = style::value(e.name())});
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

fn print_error_details(
    logger: Print<Bullet<Stderr>>,
    error: &impl Display,
) -> Print<Bullet<Stderr>> {
    logger
        .bullet(style::important("DEBUG INFO:"))
        .sub_bullet(error.to_string())
        .done()
}
