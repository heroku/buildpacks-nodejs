use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::npm_registry::{PackagePackument, packument_layer, resolve_package_packument};
use crate::utils::vrs::{Requirement, Version};
use crate::{BuildpackBuildContext, BuildpackResult, utils};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use indoc::formatdoc;
use libcnb::Env;
use libcnb::data::layer_name;
use libcnb::data::store::Store;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
    UncachedLayerDefinition,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

pub(crate) fn resolve_pnpm_package_packument(
    context: &BuildpackBuildContext,
    requirement: &Requirement,
) -> BuildpackResult<PackagePackument> {
    resolve_package_packument(
        &packument_layer(layer_name!("pnpm_packument"), context, "pnpm")?,
        requirement,
    )
    .map_err(Into::into)
}

pub(crate) fn install_pnpm(
    context: &BuildpackBuildContext,
    env: &mut Env,
    pnpm_packument: &PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    utils::npm_registry::install_package_layer(
        layer_name!("pnpm"),
        context,
        env,
        pnpm_packument,
        node_version,
    )
    .map_err(Into::into)
}

pub(crate) fn install_dependencies(
    context: &BuildpackBuildContext,
    env: &Env,
    store: &mut Store,
    version: &Version,
) -> BuildpackResult<()> {
    print::bullet("Setting up pnpm dependency store");

    let pnpm_store_dir = create_store_directory(context)?;
    set_store_dir_config(env, &pnpm_store_dir, version)?;

    let pnpm_virtual_store_dir = create_virtual_store_directory(context)?;
    set_virtual_store_dir_config(env, &pnpm_virtual_store_dir, version)?;

    print::bullet("Installing dependencies");
    print::sub_stream_cmd(
        Command::new("pnpm")
            .args(["install", "--frozen-lockfile"])
            .envs(env),
    )
    .map_err(|e| create_pnpm_install_command_error(&e))?;

    maybe_prune_store_directory(env, store)?;

    Ok(())
}

fn create_store_directory(context: &BuildpackBuildContext) -> BuildpackResult<PathBuf> {
    let new_metadata = PnpmCacheDirectoryLayerMetadata {
        layer_version: PNPM_CACHE_DIRECTORY_LAYER_VERSION.to_string(),
    };

    let pnpm_cache_layer = context.cached_layer(
        layer_name!("addressable"),
        CachedLayerDefinition {
            build: true,
            launch: false,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &PnpmCacheDirectoryLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match pnpm_cache_layer.state {
        LayerState::Restored { .. } => {
            print::sub_bullet("Restoring pnpm content-addressable store from cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                print::sub_bullet("Cached pnpm content-addressable store has expired");
            }
            print::sub_bullet("Creating new pnpm content-addressable store");
            pnpm_cache_layer.write_metadata(new_metadata)?;
        }
    }

    Ok(pnpm_cache_layer.path())
}

const PNPM_CACHE_DIRECTORY_LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PnpmCacheDirectoryLayerMetadata {
    layer_version: String,
}

fn set_store_dir_config(
    env: &Env,
    store_dir: &Path,
    version: &Version,
) -> Result<(), ErrorMessage> {
    Command::new("pnpm")
        .args([
            "config",
            "set",
            match version.major() {
                major if major < 10 => "store-dir",
                _ => "storeDir",
            },
            &store_dir.to_string_lossy(),
        ])
        .envs(env)
        .named_output()
        .map(|_| ())
        .map_err(|e| create_set_store_dir_config_error(&e))
}

fn create_set_store_dir_config_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .id("package_manager/pnpm/set_store_dir_config")
        .error_type(ErrorType::Internal)
        .header("Failed to configure pnpm store dir")
        .body(formatdoc! { "
            An unexpected error occurred while configuring the store directory for pnpm. This is the location \
            on disk where pnpm saves all packages.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_virtual_store_directory(context: &BuildpackBuildContext) -> BuildpackResult<PathBuf> {
    print::sub_bullet("Creating pnpm virtual store");

    let pnpm_virtual_store_layer = context.uncached_layer(
        layer_name!("virtual"),
        UncachedLayerDefinition {
            build: true,
            launch: true,
        },
    )?;

    let virtual_store_dir = pnpm_virtual_store_layer.path().join("store");

    fs::create_dir(&virtual_store_dir)
        .map_err(|e| create_virtual_store_error(&virtual_store_dir, &e))?;

    // Install a symlink from {virtual_layer}/node_modules to {app_dir}/node_modules, so that dependencies in
    // {virtual_layer}/store/ can find their dependencies via the Node module loader's ancestor directory traversal.
    // https://nodejs.org/api/modules.html#loading-from-node_modules-folders
    // https://nodejs.org/api/esm.html#resolution-and-loading-algorithm
    if let Err(error) = std::os::unix::fs::symlink(
        context.app_dir.join("node_modules"),
        pnpm_virtual_store_layer.path().join("node_modules"),
    ) && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        Err(create_node_modules_symlink_error(&error))?;
    }

    Ok(virtual_store_dir)
}

fn create_virtual_store_error(path: &Path, error: &std::io::Error) -> ErrorMessage {
    let path = file_value(path);
    error_message()
        .id("package_manager/pnpm/create_virtual_store_directory")
        .error_type(ErrorType::Internal)
        .header("Failed to create directory")
        .body(formatdoc! { "
            An unexpected I/O error occurred while creating the directory at {path}.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_node_modules_symlink_error(error: &std::io::Error) -> ErrorMessage {
    error_message()
        .id("package_manager/pnpm/create_node_modules_symlink")
        .error_type(ErrorType::Internal)
        .header("Failed to create pnpm symlink")
        .body(formatdoc! { "
            An unexpected error occurred while creating the symlink for pnpm. This is the location \
            on disk where pnpm saves all packages.
        " })
        .debug_info(error.to_string())
        .create()
}

fn set_virtual_store_dir_config(
    env: &Env,
    virtual_store_dir: &Path,
    version: &Version,
) -> Result<(), ErrorMessage> {
    Command::new("pnpm")
        .args([
            "config",
            "set",
            match version.major() {
                major if major < 10 => "virtual-store-dir",
                _ => "virtualStoreDir",
            },
            &virtual_store_dir.to_string_lossy(),
        ])
        .envs(env)
        .named_output()
        .map(|_| ())
        .map_err(|e| create_set_virtual_store_dir_config_error(&e))
}

fn create_set_virtual_store_dir_config_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .id("package_manager/pnpm/set_virtual_store_dir_config")
        .error_type(ErrorType::Internal)
        .header("Failed to configure pnpm virtual store dir")
        .body(formatdoc! { "
            An unexpected error occurred while configuring the store directory for pnpm. This is the directory \
            where pnpm links all installed packages from the store.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_pnpm_install_command_error(error: &fun_run::CmdError) -> ErrorMessage {
    let pnpm_install = style::value(error.name());
    error_message()
        .id("package_manager/pnpm/install")
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header("Failed to install Node modules")
        .body(formatdoc! { "
            The Heroku Node.js buildpack uses the command {pnpm_install} to install your Node modules. This command \
            failed and the buildpack cannot continue. This error can occur due to an unstable network connection. \
            See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
            - Check the status of the upstream Node module repository service at https://status.npmjs.org/
        " })
        .debug_info(error.to_string())
        .create()
}

fn maybe_prune_store_directory(env: &Env, store: &mut Store) -> BuildpackResult<()> {
    let cache_use_count_key = "cache_use_count";
    let cache_prune_interval = 40;

    #[allow(clippy::cast_possible_truncation)]
    let cache_use_count = store.metadata.get(cache_use_count_key).map_or(0, |v| {
        v.as_float().map_or(cache_prune_interval, |f| f as i64)
    });

    if cache_use_count.rem_euclid(cache_prune_interval) == 0 {
        print::bullet("Pruning unused dependencies from pnpm content-addressable store");
        print::sub_stream_cmd(Command::new("pnpm").args(["store", "prune"]).envs(env))
            .map_err(|e| create_prune_store_directory_error(&e))?;
    }

    #[allow(clippy::cast_precision_loss)]
    store.metadata.insert(
        cache_use_count_key.into(),
        toml::Value::from((cache_use_count + 1) as f64),
    );

    Ok(())
}

fn create_prune_store_directory_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .id("package_manager/pnpm/prune_store_directory")
        .error_type(ErrorType::Internal)
        .header("Failed to prune packages from the store directory")
        .body(formatdoc! { "
            The Heroku Node.js buildpack periodically cleans up the store directory to remove \
            packages that are no longer in use from the cache. An unexpected error occurred \
            during this operation.
        " })
        .debug_info(error.to_string())
        .create()
}

pub(crate) fn run_script(name: impl AsRef<str>, env: &Env) -> Command {
    let mut command = Command::new("pnpm");
    command.args(["run", name.as_ref()]);
    command.envs(env);
    command
}

pub(crate) fn prune_dev_dependencies(
    context: &BuildpackBuildContext,
    env: &Env,
    version: &Version,
    on_prune_command_error: impl FnOnce(&fun_run::CmdError) -> ErrorMessage,
) -> Result<(), ErrorMessage> {
    let has_workspace_file = ["pnpm-workspace.yaml", "pnpm-workspace.yml"]
        .iter()
        .any(|file| context.app_dir.join(file).exists());

    if has_workspace_file {
        print::sub_bullet(format!(
            "Skipping because pruning is not supported for pnpm workspaces ({})",
            style::url("https://pnpm.io/cli/prune")
        ));
        return Ok(());
    }

    let mut cmd = Command::new("pnpm");
    cmd.envs(env);
    cmd.args(["prune", "--prod"]);

    if PRUNE_DEV_DEPENDENCIES_MIN_VERSION.satisfies(version) {
        cmd.arg("--ignore-scripts");
        return print::sub_stream_cmd(cmd)
            .map(|_| ())
            .map_err(|e| on_prune_command_error(&e));
    }

    let package_json =
        crate::package_json::PackageJson::try_from(context.app_dir.join("package.json"))?;

    if ["pnpm:devPreinstall", "preinstall", "postinstall", "prepare"]
        .iter()
        .any(|script| package_json.script(script).is_some())
    {
        print::warning(formatdoc! { "
            Pruning skipped due to presence of lifecycle scripts

            The version of pnpm used ({version}) will execute the following lifecycle scripts \
            declared in package.json during pruning which can cause build failures:
            - pnpm:devPreinstall
            - preinstall
            - install
            - postinstall
            - prepare

            Since pruning can't be done safely for your build, it will be skipped. To fix this you \
            must upgrade your version of pnpm to 8.15.6 or higher.
        "});
        return Ok(());
    }

    print::sub_stream_cmd(cmd)
        .map(|_| ())
        .map_err(|e| on_prune_command_error(&e))
}

static PRUNE_DEV_DEPENDENCIES_MIN_VERSION: LazyLock<Requirement> = LazyLock::new(|| {
    Requirement::parse(">8.15.6")
        .expect("Prune dev dependencies min version requirement should be valid")
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_cmd_error};

    #[test]
    fn set_store_dir_config_error() {
        assert_error_snapshot(&create_set_store_dir_config_error(&create_cmd_error(
            "pnpm config set store-dir /some/dir",
        )));
    }

    #[test]
    fn set_virtual_store_dir_config_error() {
        assert_error_snapshot(&create_set_virtual_store_dir_config_error(
            &create_cmd_error("pnpm config set virtual-store-dir /some/dir"),
        ));
    }

    #[test]
    fn virtual_store_directory_error() {
        assert_error_snapshot(&create_virtual_store_error(
            &PathBuf::from("/layers/some/dir"),
            &std::io::Error::other("Insufficient permissions"),
        ));
    }

    #[test]
    fn node_modules_symlink_error() {
        assert_error_snapshot(&create_node_modules_symlink_error(&std::io::Error::other(
            "Target directory does not exist",
        )));
    }

    #[test]
    fn prune_store_directory_error() {
        assert_error_snapshot(&create_prune_store_directory_error(&create_cmd_error(
            "pnpm store prune",
        )));
    }
}
