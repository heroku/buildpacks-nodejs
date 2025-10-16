use crate::utils::error_handling::ErrorType::Internal;
use crate::utils::error_handling::{ErrorMessage, error_message};
use crate::utils::npm_registry;
use crate::utils::vrs::{Requirement, Version, VersionCommandError};
use crate::{BuildpackBuildContext, BuildpackResult};
use fun_run::CommandWithName;
use indoc::formatdoc;
use libcnb::Env;
use libcnb::data::layer_name;
use std::process::Command;

pub(crate) fn resolve_npm_package_packument(
    context: &BuildpackBuildContext,
    requirement: &Requirement,
) -> BuildpackResult<npm_registry::PackagePackument> {
    npm_registry::resolve_package_packument(
        &npm_registry::packument_layer(layer_name!("npm_packument"), context, "npm")?,
        requirement,
    )
    .map_err(Into::into)
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

pub(crate) fn install_npm(
    context: &BuildpackBuildContext,
    env: &mut Env,
    npm_packument: &npm_registry::PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    npm_registry::install_package_layer(
        layer_name!("npm_engine"),
        context,
        env,
        npm_packument,
        node_version,
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_cmd_error};

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
