use crate::utils::error_handling::ErrorType::{Internal, UserFacing};
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use crate::utils::npm_registry::{
    PackagePackument, PackumentLayerError, packument_layer, resolve_package_packument,
};
use crate::utils::vrs::{Requirement, Version, VersionCommandError};
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::style;
use fun_run::CommandWithName;
use indoc::formatdoc;
use libcnb::Env;
use std::process::Command;

pub(crate) fn resolve_npm_package_packument(
    context: &BuildpackBuildContext,
    requirement: &Requirement,
) -> BuildpackResult<PackagePackument> {
    let npm_packument = packument_layer(context, "npm", |error| {
        create_npm_packument_layer_error(&error)
    })?;

    let npm_package_packument = resolve_package_packument(&npm_packument, requirement)
        .ok_or_else(|| create_resolve_npm_package_packument_error(requirement))?;

    Ok(npm_package_packument)
}

fn create_npm_packument_layer_error(error: &PackumentLayerError) -> ErrorMessage {
    let npm = style::value("npm");
    let npm_status_url = style::url("https://status.npmjs.org/");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header(format!("Failed to load available {npm} versions"))
        .body(formatdoc! { "
            An unexpected error occurred while loading the available {npm} versions. This error can \
            occur due to an unstable network connection or an issue with the npm registry.

            Suggestions:
            - Check the npm status page for any ongoing incidents ({npm_status_url})
        "})
        .debug_info(error.to_string())
        .create()
}

fn create_resolve_npm_package_packument_error(requirement: &Requirement) -> ErrorMessage {
    let npm = style::value("npm");
    let requested_version = style::value(requirement.to_string());
    let npm_releases_url = style::url("https://www.npmjs.com/package/npm?activeTab=versions");
    let npm_show_command = style::value(format!("npm show 'npm@{requirement}' versions"));
    let package_json = style::value("package.json");
    let engines_key = style::value("engines.npm");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!("Error resolving requested {npm} version {requested_version}"))
        .body(formatdoc! { "
                The requested npm version could not be resolved to a known release in this buildpack's \
                inventory of npm releases.

                Suggestions:
                - Confirm if this is a valid npm release at {npm_releases_url} or by running {npm_show_command}
                - Update the {engines_key} field in {package_json} to a single version or version range that \
                includes a published {npm} version.
            " })
        .create()
}

pub(crate) fn get_version(env: &Env) -> BuildpackResult<Version> {
    Command::new("npm")
        .envs(env)
        .arg("--version")
        .named_output()
        .try_into()
        .map_err(|e| create_get_npm_version_command_error(&e).into())
}

fn create_get_npm_version_command_error(error: &VersionCommandError) -> ErrorMessage {
    match error {
        VersionCommandError::Command(e) => error_message()
            .error_type(Internal)
            .header("Failed to determine npm version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current npm version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        VersionCommandError::Parse(stdout, e) => error_message()
            .error_type(Internal)
            .header("Failed to parse npm version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing npm version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_cmd_error};

    #[test]
    fn packument_layer_error() {
        assert_error_snapshot(&create_npm_packument_layer_error(
            &PackumentLayerError::ReadPackument(std::io::Error::other("Insufficient permissions")),
        ));
    }

    #[test]
    fn resolve_package_packument_error() {
        assert_error_snapshot(&create_resolve_npm_package_packument_error(
            &Requirement::parse("1.2.3").unwrap(),
        ));
    }

    #[test]
    fn version_parse_error() {
        assert_error_snapshot(&create_get_npm_version_command_error(
            &VersionCommandError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    #[test]
    fn version_command_error() {
        assert_error_snapshot(&create_get_npm_version_command_error(
            &VersionCommandError::Command(create_cmd_error("npm --version")),
        ));
    }
}
