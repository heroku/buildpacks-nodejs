use crate::utils::error_handling::ErrorType::Internal;
use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use crate::utils::npm_registry;
use crate::utils::vrs::{Requirement, Version, VersionCommandError};
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use indoc::formatdoc;
use libcnb::Env;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
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

pub(crate) fn install_npm_dependencies(
    context: &BuildpackBuildContext,
    env: &Env,
    npm_version: &Version,
) -> BuildpackResult<()> {
    print::bullet("Installing node modules");
    print::sub_bullet(format!(
        "Using npm version {}",
        style::value(npm_version.to_string())
    ));

    let cache_dir = create_cache_directory(context)?;

    print::sub_bullet("Configuring npm cache directory");
    Command::new("npm")
        .args([
            "config",
            "set",
            "cache",
            &cache_dir.to_string_lossy(),
            "--global",
        ])
        .envs(env)
        .named_output()
        .map_err(|e| create_set_npm_cache_directory_command_error(&e))?;

    print::sub_stream_cmd(Command::new("npm").args(["ci"]).envs(env))
        .map_err(|e| create_npm_install_error(&e))?;

    Ok(())
}

fn create_cache_directory(context: &BuildpackBuildContext) -> BuildpackResult<PathBuf> {
    let new_metadata = NpmCacheDirectoryLayerMetadata {
        layer_version: NPM_CACHE_DIRECTORY_LAYER_VERSION.to_string(),
    };

    let npm_cache_layer = context.cached_layer(
        layer_name!("npm_cache"),
        CachedLayerDefinition {
            build: true,
            launch: false,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &NpmCacheDirectoryLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match npm_cache_layer.state {
        LayerState::Restored { .. } => {
            print::sub_bullet("Restoring npm cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                print::sub_bullet("Restoring npm cache");
            }
            print::sub_bullet("Creating npm cache");
            npm_cache_layer.write_metadata(new_metadata)?;
        }
    }

    Ok(npm_cache_layer.path().clone())
}

fn create_set_npm_cache_directory_command_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .error_type(Internal)
        .header("Failed to set the npm cache directory")
        .body("An unexpected error occurred while setting the npm cache directory.")
        .debug_info(error.to_string())
        .create()
}

fn create_npm_install_error(error: &fun_run::CmdError) -> ErrorMessage {
    let npm_install = style::value(error.name());
    error_message()
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header("Failed to install Node modules")
        .body(formatdoc! { "
            The Heroku Node.js buildpack uses the command {npm_install} to install your Node modules. This command \
            failed and the buildpack cannot continue. This error can occur due to an unstable network connection. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
            - Check the status of the upstream Node module repository service at https://status.npmjs.org/
        " })
        .debug_info(error.to_string())
        .create()
}

const NPM_CACHE_DIRECTORY_LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NpmCacheDirectoryLayerMetadata {
    layer_version: String,
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

    #[test]
    fn set_npm_cache_directory_command_error() {
        assert_error_snapshot(&create_set_npm_cache_directory_command_error(
            &create_cmd_error("npm config set cache /some/dir --global"),
        ));
    }

    #[test]
    fn npm_install_error() {
        assert_error_snapshot(&create_npm_install_error(&create_cmd_error("npm ci")));
    }
}
