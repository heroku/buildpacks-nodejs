use crate::cleanup::{CleanupTask, NodeGypArtifactLocation};
use crate::package_json::PackageJson;
use crate::utils::build_env::node_gyp_env;
use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::npm_registry::{
    InstallPackageLayerError, PackagePackument, apply_package_layer_env, cached_package_layer,
    exclude_package_docs, extract_package_tarball, install_package_layer_metadata,
    link_package_bins, packument_layer, resolve_package_packument,
};
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
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use nodejs_data::{Version, VersionRange};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

pub(crate) fn resolve_pnpm_package_packument(
    context: &BuildpackBuildContext,
    requirement: &VersionRange,
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
    // Starting with pnpm 12, the published `pnpm` npm package is a thin wrapper: its tarball ships
    // a placeholder `pnpm` bin plus a `preinstall` script (`install.js`) that hard-links the real,
    // platform-specific native binary — distributed separately as `@pnpm/exe.<target>` optional
    // dependencies — over that placeholder. The buildpack installs from the tarball directly and
    // never runs npm lifecycle scripts, so pnpm 12+ needs its own install path that relinks the
    // native binary once the wrapper is extracted.
    if pnpm_packument.version.major() >= 12 {
        install_pnpm_layer_with_native_binary_overlay(context, env, pnpm_packument, node_version)
    } else {
        utils::npm_registry::install_package_layer(
            layer_name!("pnpm"),
            context,
            env,
            pnpm_packument,
            node_version,
        )?;
        Ok(())
    }
}

/// Installs the pnpm 12+ wrapper package into a cached layer, then overlays the platform-specific
/// native binary over the wrapper's placeholder `pnpm` bin. This is the same composition as the
/// generic `install_package_layer` with the native-binary overlay inserted inline in the
/// layer-population branch: it runs only when the layer is populated (never on a cache hit) and
/// before the layer metadata is written, so a failed overlay leaves the layer incomplete and the
/// next build repopulates it rather than caching a broken placeholder.
fn install_pnpm_layer_with_native_binary_overlay(
    context: &BuildpackBuildContext,
    env: &mut Env,
    pnpm_packument: &PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    let new_metadata = install_package_layer_metadata(context, pnpm_packument, node_version);
    let layer = cached_package_layer(layer_name!("pnpm"), context, &new_metadata)?;

    if matches!(layer.state, LayerState::Empty { .. }) {
        extract_package_tarball(
            &pnpm_packument.dist.tarball,
            &layer.path(),
            exclude_package_docs,
        )
        .map_err(|e| InstallPackageLayerError::Download(Box::new(pnpm_packument.clone()), e))?;

        link_package_bins(pnpm_packument, &layer.path())?;

        // Overlay the native binary over the wrapper's placeholder before the layer is marked
        // complete, so a failed overlay is never captured in a cached layer. The native binary is
        // pinned to the wrapper's exact version, so it is only resolved and downloaded while
        // (re)populating the layer — a cache hit needs neither.
        let native_binary_packument =
            resolve_pnpm_native_binary_packument(context, &pnpm_packument.version)?;
        install_pnpm_native_binary(&native_binary_packument, &layer.path())?;

        layer
            .write_metadata(new_metadata)
            .map_err(|e| InstallPackageLayerError::Layer(Box::new(e)))?;
    }

    apply_package_layer_env(&layer, env)?;

    Ok(())
}

/// Resolves the packument for the `@pnpm/exe.<target>` package that ships the native pnpm binary
/// for the current build target, pinned to the same version as the wrapper package.
fn resolve_pnpm_native_binary_packument(
    context: &BuildpackBuildContext,
    version: &Version,
) -> BuildpackResult<PackagePackument> {
    let package_name = pnpm_native_binary_package_name(&context.target.os, &context.target.arch)?;
    let requirement = VersionRange::parse(&version.to_string())
        .expect("A pnpm version should be a valid version requirement");
    resolve_package_packument(
        &packument_layer(layer_name!("pnpm_exe_packument"), context, &package_name)?,
        &requirement,
    )
    .map_err(Into::into)
}

/// Maps the build target to the `@pnpm/exe.<target>` package that ships its native binary. Heroku
/// base images are glibc-based, so the musl variants are never selected.
fn pnpm_native_binary_package_name(os: &str, arch: &str) -> BuildpackResult<String> {
    match (os, arch) {
        ("linux", "amd64") => Ok("@pnpm/exe.linux-x64".to_string()),
        ("linux", "arm64") => Ok("@pnpm/exe.linux-arm64".to_string()),
        _ => Err(create_pnpm_unsupported_target_error(os, arch).into()),
    }
}

/// Downloads the native pnpm binary package and moves its executable over the wrapper's placeholder
/// bin. The wrapper's `bin/pnpm` symlink already points at `<layer>/pnpm`, so replacing that file
/// activates the native binary.
fn install_pnpm_native_binary(
    native_binary_packument: &PackagePackument,
    pnpm_layer_dir: &Path,
) -> BuildpackResult<()> {
    let scratch_dir = pnpm_layer_dir.join(".native-binary");
    extract_package_tarball(
        &native_binary_packument.dist.tarball,
        &scratch_dir,
        // The package ships the executable alongside license files; only the binary is needed.
        |path| path != Path::new("pnpm"),
    )
    .map_err(|e| {
        InstallPackageLayerError::Download(Box::new(native_binary_packument.clone()), e)
    })?;

    let native_pnpm_binary = scratch_dir.join("pnpm");
    let original_pnpm_binary = pnpm_layer_dir.join("pnpm");
    fs::rename(&native_pnpm_binary, &original_pnpm_binary)
        .map_err(|e| create_pnpm_native_binary_install_error(&original_pnpm_binary, &e))?;

    fs::remove_dir_all(&scratch_dir)
        .map_err(|e| create_pnpm_native_binary_install_error(&scratch_dir, &e))?;

    Ok(())
}

fn create_pnpm_unsupported_target_error(os: &str, arch: &str) -> ErrorMessage {
    let target = style::value(format!("{os}-{arch}"));
    error_message()
        .id("package_manager/pnpm/unsupported_native_binary_target")
        .error_type(ErrorType::Internal)
        .header("Unsupported pnpm target")
        .body(formatdoc! { "
            pnpm 12 and later ship their native binary as a platform-specific package, but the \
            buildpack does not know which package to use for the current build target ({target}).
        " })
        .create()
}

fn create_pnpm_native_binary_install_error(path: &Path, error: &std::io::Error) -> ErrorMessage {
    let path = file_value(path);
    error_message()
        .id("package_manager/pnpm/install_native_binary")
        .error_type(ErrorType::Internal)
        .header("Failed to install the pnpm native binary")
        .body(formatdoc! { "
            An unexpected I/O error occurred while installing the pnpm native binary at {path}.
        " })
        .debug_info(error.to_string())
        .create()
}

pub(crate) fn install_dependencies(
    context: &BuildpackBuildContext,
    env: &mut Env,
    store: &mut Store,
    version: &Version,
) -> BuildpackResult<()> {
    print::bullet("Setting up pnpm dependency store");

    let store_dir = create_store_directory(context, env, version)?;
    let virtual_store_dir = create_virtual_store_directory(context, env, version)?;

    verify_pnpm_config(env, "store-dir", &store_dir);
    verify_pnpm_config(env, "virtual-store-dir", &virtual_store_dir);

    print::bullet("Installing dependencies");
    print::sub_stream_cmd(
        Command::new("pnpm")
            .args(["install", "--frozen-lockfile"])
            .envs(&*env)
            .envs(node_gyp_env()),
    )
    .map_err(|e| create_pnpm_install_command_error(&e))?;

    maybe_prune_store_directory(&*env, store)?;

    context.register_cleanup(CleanupTask::PnpmModulesYaml(context.app_dir.clone()));

    Ok(())
}

fn create_store_directory(
    context: &BuildpackBuildContext,
    env: &mut Env,
    version: &Version,
) -> BuildpackResult<PathBuf> {
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

    let store_dir = pnpm_cache_layer.path();

    // pnpm 11 dropped npm_config_* env var support in favor of pnpm_config_*
    let env_var_name = if version.major() >= 11 {
        "pnpm_config_store_dir"
    } else {
        "npm_config_store_dir"
    };
    let mut layer_env = LayerEnv::new();
    layer_env.insert(
        Scope::Build,
        ModificationBehavior::Override,
        env_var_name,
        &store_dir,
    );
    pnpm_cache_layer.write_env(layer_env)?;
    env.clone_from(&pnpm_cache_layer.read_env()?.apply(Scope::Build, env));

    Ok(store_dir)
}

const PNPM_CACHE_DIRECTORY_LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PnpmCacheDirectoryLayerMetadata {
    layer_version: String,
}

fn create_virtual_store_directory(
    context: &BuildpackBuildContext,
    env: &mut Env,
    version: &Version,
) -> BuildpackResult<PathBuf> {
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

    // Register virtual layer for cleanup of non-deterministic Makefiles
    context.register_cleanup(CleanupTask::NodeGypMakefiles(
        NodeGypArtifactLocation::PnpmVirtualStore(pnpm_virtual_store_layer.path().clone()),
    ));

    // pnpm 11 dropped npm_config_* env var support for this key in favor of pnpm_config_*
    let env_var_name = if version.major() >= 11 {
        "pnpm_config_virtual_store_dir"
    } else {
        "npm_config_virtual_store_dir"
    };
    let mut layer_env = LayerEnv::new();
    layer_env.insert(
        Scope::All,
        ModificationBehavior::Override,
        env_var_name,
        &virtual_store_dir,
    );
    pnpm_virtual_store_layer.write_env(layer_env)?;
    env.clone_from(
        &pnpm_virtual_store_layer
            .read_env()?
            .apply(Scope::Build, env),
    );

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

fn verify_pnpm_config(env: &Env, config_key: &str, expected: &Path) {
    let matches = Command::new("pnpm")
        .args(["config", "get", config_key])
        .envs(env)
        .named_output()
        .is_ok_and(|output| output.stdout_lossy().trim() == expected.to_string_lossy().as_ref());

    if !matches {
        let expected = file_value(expected);
        print::warning(formatdoc! { "
            pnpm config {config_key} is not set to the expected buildpack-managed directory ({expected}). \
            This may result in slower builds due to reduced caching effectiveness.
        "});
    }
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

fn create_pnpm_workspace_read_error(workspace_file: &Path, error: &std::io::Error) -> ErrorMessage {
    let file = file_value(workspace_file);
    error_message()
        .id("package_manager/pnpm/read_workspace_file")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error reading {file}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {file} to determine pnpm workspace configuration but \
            the file can't be read.

            Suggestions:
            - Ensure the file has read permissions.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_pnpm_workspace_parse_error(error: &yaml_rust2::ScanError) -> ErrorMessage {
    let file = file_value("pnpm-workspace.yaml");
    let yaml_spec_url = style::url("https://yaml.org/spec/1.2.2/");
    error_message()
        .id("package_manager/pnpm/parse_workspace_file")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error parsing {file}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {file} to determine pnpm workspace configuration but \
            the file isn't valid YAML.

            Suggestions:
            - Ensure the file follows the YAML format described at {yaml_spec_url}
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_pnpm_workspace_multiple_documents_error() -> ErrorMessage {
    let file = file_value("pnpm-workspace.yaml");
    let hyphens = style::value("---");
    error_message()
        .id("package_manager/pnpm/multiple_documents_in_workspace_file")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Multiple YAML documents found in {file}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {file} to determine pnpm workspace configuration but \
            the file contains multiple YAML documents. There must only be a single document in this file.

            YAML documents are separated by a line containing three hyphens ({hyphens}).

            Suggestions:
            - Ensure the file has only a single document.
        " })
        .create()
}

fn create_delete_node_modules_error(path: &Path, error: &std::io::Error) -> ErrorMessage {
    let path = file_value(path);
    error_message()
        .id("package_manager/pnpm/delete_node_modules")
        .error_type(ErrorType::Internal)
        .header("Failed to delete node_modules directory")
        .body(formatdoc! { "
            An unexpected I/O error occurred while deleting the directory at {path}.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_list_workspace_packages_error(error: &fun_run::CmdError) -> ErrorMessage {
    let pnpm_list = style::command(error.name());
    error_message()
        .id("package_manager/pnpm/list_workspace_packages")
        .error_type(ErrorType::Internal)
        .header("Failed to list workspace packages")
        .body(formatdoc! { "
            An unexpected error occurred while running {pnpm_list} to determine workspace package locations.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_parse_pnpm_list_output_error(error: &serde_json::Error) -> ErrorMessage {
    error_message()
        .id("package_manager/pnpm/parse_list_output")
        .error_type(ErrorType::Internal)
        .header("Failed to parse pnpm list output")
        .body(formatdoc! { "
            An unexpected error occurred while parsing the JSON output from pnpm list command.
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
    let is_workspace = match read_pnpm_workspace(&context.app_dir) {
        None => false,
        Some(Ok(workspace)) => workspace.has_packages(),
        Some(Err(error)) => return Err(error),
    };

    if is_workspace {
        // Get workspace package locations from pnpm (includes root + all workspace packages)
        let workspace_projects = get_workspace_package_dirs(&context.app_dir, env)?;

        for project in &workspace_projects {
            if contains_pnpm_lifecycle_script(project.join("package.json"))? {
                let package_json = file_value(project.join("package.json"));
                print::warning(formatdoc! { "
                    Pruning skipped due to presence of lifecycle scripts
        
                    Lifecycle scripts were detected in {package_json}. Due to how workspace pruning \
                    in pnpm operates, it will execute the following lifecycle scripts during reinstallation \
                    of production dependencies which can cause build failures:
                    - pnpm:devPreinstall
                    - preinstall
                    - install
                    - postinstall
                    - prepare

                    Since pruning can't be done safely for your build, it will be skipped.
                "});
                return Ok(());
            }
        }

        delete_workspace_node_modules(&workspace_projects)?;

        print::sub_stream_cmd(
            Command::new("pnpm")
                .args(["install", "--prod", "--frozen-lockfile"])
                .envs(env),
        )
        .map(|_| ())
        .map_err(|e| on_prune_command_error(&e))?;

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

    if contains_pnpm_lifecycle_script(context.app_dir.join("package.json"))? {
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

fn contains_pnpm_lifecycle_script(package_json_path: PathBuf) -> Result<bool, ErrorMessage> {
    let package_json = PackageJson::try_from(package_json_path)?;
    Ok(
        ["pnpm:devPreinstall", "preinstall", "postinstall", "prepare"]
            .iter()
            .any(|script| package_json.script(script).is_some()),
    )
}

static PRUNE_DEV_DEPENDENCIES_MIN_VERSION: LazyLock<VersionRange> = LazyLock::new(|| {
    VersionRange::parse(">8.15.6")
        .expect("Prune dev dependencies min version requirement should be valid")
});

fn delete_workspace_node_modules(workspace_dirs: &[PathBuf]) -> Result<(), ErrorMessage> {
    // Delete node_modules in root and each workspace package
    for package_dir in workspace_dirs {
        let node_modules = package_dir.join("node_modules");
        match fs::remove_dir_all(&node_modules) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(create_delete_node_modules_error(&node_modules, &error)),
        }
    }
    Ok(())
}

fn get_workspace_package_dirs(app_dir: &Path, env: &Env) -> Result<Vec<PathBuf>, ErrorMessage> {
    let output = Command::new("pnpm")
        .args(["list", "--depth", "-1", "--json", "--recursive"])
        .current_dir(app_dir)
        .envs(env)
        .named_output()
        .map_err(|e| create_list_workspace_packages_error(&e))?;

    parse_pnpm_list_output(&output.stdout_lossy())
}

fn parse_pnpm_list_output(json_output: &str) -> Result<Vec<PathBuf>, ErrorMessage> {
    let packages: Vec<serde_json::Value> =
        serde_json::from_str(json_output).map_err(|e| create_parse_pnpm_list_output_error(&e))?;

    Ok(packages
        .into_iter()
        .filter_map(|pkg| pkg["path"].as_str().map(PathBuf::from))
        .collect())
}

fn read_pnpm_workspace(app_dir: &Path) -> Option<Result<PnpmWorkspace, ErrorMessage>> {
    let workspace_file = app_dir.join("pnpm-workspace.yaml");

    let contents = match std::fs::read_to_string(&workspace_file) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            return Some(Err(create_pnpm_workspace_read_error(
                &workspace_file,
                &error,
            )));
        }
    };

    match yaml_rust2::YamlLoader::load_from_str(&contents) {
        Ok(docs) if docs.len() == 1 => Some(Ok(PnpmWorkspace(docs.into_iter().next()))),
        Ok(docs) if docs.is_empty() => Some(Ok(PnpmWorkspace(None))),
        Ok(_) => Some(Err(create_pnpm_workspace_multiple_documents_error())),
        Err(error) => Some(Err(create_pnpm_workspace_parse_error(&error))),
    }
}

#[derive(Debug)]
struct PnpmWorkspace(Option<yaml_rust2::Yaml>);

impl PnpmWorkspace {
    fn has_packages(&self) -> bool {
        self.0
            .as_ref()
            .and_then(|doc| doc["packages"].as_vec())
            .is_some_and(|packages| !packages.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_cmd_error};

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

    #[test]
    fn pnpm_workspace_read_error() {
        assert_error_snapshot(&create_pnpm_workspace_read_error(
            &PathBuf::from("/app/pnpm-workspace.yaml"),
            &std::io::Error::other("Permission denied"),
        ));
    }

    #[test]
    fn pnpm_workspace_parse_error() {
        let yaml_error =
            yaml_rust2::YamlLoader::load_from_str("invalid: yaml: content:").unwrap_err();
        assert_error_snapshot(&create_pnpm_workspace_parse_error(&yaml_error));
    }

    #[test]
    fn pnpm_workspace_multiple_documents_error() {
        assert_error_snapshot(&create_pnpm_workspace_multiple_documents_error());
    }

    #[test]
    fn delete_node_modules_error() {
        assert_error_snapshot(&create_delete_node_modules_error(
            &PathBuf::from("/app/packages/foo/node_modules"),
            &std::io::Error::other("Device is busy"),
        ));
    }

    #[test]
    fn list_workspace_packages_error() {
        assert_error_snapshot(&create_list_workspace_packages_error(&create_cmd_error(
            "pnpm list --depth -1 --json --recursive",
        )));
    }

    #[test]
    fn parse_pnpm_list_output_error() {
        let json_error =
            serde_json::from_str::<Vec<serde_json::Value>>("invalid json").unwrap_err();
        assert_error_snapshot(&create_parse_pnpm_list_output_error(&json_error));
    }

    #[test]
    fn pnpm_unsupported_target_error() {
        assert_error_snapshot(&create_pnpm_unsupported_target_error("windows", "arm64"));
    }

    #[test]
    fn pnpm_native_binary_install_error() {
        assert_error_snapshot(&create_pnpm_native_binary_install_error(
            &PathBuf::from("/layers/heroku_nodejs/pnpm/pnpm"),
            &std::io::Error::other("Permission denied"),
        ));
    }
}
