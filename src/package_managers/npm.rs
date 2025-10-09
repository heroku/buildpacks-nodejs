use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use crate::utils::npm_registry::{
    PackagePackument, PackumentLayerError, packument_layer, resolve_package_packument,
};
use crate::utils::vrs::Requirement;
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::style;
use indoc::formatdoc;

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

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use insta::{assert_snapshot, with_settings};
    use test_support::test_name;

    #[test]
    fn test_create_npm_packument_layer_error() {
        assert_error_snapshot(&create_npm_packument_layer_error(
            &PackumentLayerError::ReadPackument(std::io::Error::other("Insufficient permissions")),
        ));
    }

    #[test]
    fn test_create_resolve_npm_package_packument_error() {
        assert_error_snapshot(&create_resolve_npm_package_packument_error(
            &Requirement::parse("1.2.3").unwrap(),
        ));
    }

    fn assert_error_snapshot(error: &ErrorMessage) {
        let error_message = strip_ansi(error.to_string());
        let test_name = format!("npm__{}", test_name().split("::").last().unwrap());
        with_settings!({
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            assert_snapshot!(test_name, error_message);
        });
    }
}
