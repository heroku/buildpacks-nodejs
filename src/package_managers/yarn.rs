use crate::cleanup::{CleanupTask, NodeGypArtifactLocation};
use crate::o11y::*;
use crate::utils::build_env::node_gyp_env;
use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::npm_registry::{PackagePackument, packument_layer, resolve_package_packument};
use crate::{BuildpackBuildContext, BuildpackResult, utils};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use indoc::formatdoc;
use libcnb::Env;
use libcnb::data::layer::LayerName;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
    UncachedLayerDefinition,
};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use nodejs_data::{Version, VersionCommandError, VersionRange};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::LazyLock;
use tracing::instrument;

static YARN_BERRY_RANGE: LazyLock<VersionRange> = LazyLock::new(|| {
    VersionRange::parse(">=2").expect("Yarn berry requirement range should be valid")
});

pub(crate) static DEFAULT_YARN_REQUIREMENT: LazyLock<VersionRange> = LazyLock::new(|| {
    VersionRange::parse("1.22.x").expect("Default Yarn requirement should be valid")
});

pub(crate) fn resolve_yarn_package_packument(
    context: &BuildpackBuildContext,
    requirement: &VersionRange,
) -> BuildpackResult<PackagePackument> {
    let (yarn_layer_name, yarn_package_name) = if requirement.allows_any(&YARN_BERRY_RANGE) {
        (
            layer_name!("yarnpkg_cli-dist_packument"),
            "@yarnpkg/cli-dist",
        )
    } else {
        (layer_name!("yarn_packument"), "yarn")
    };
    resolve_package_packument(
        &packument_layer(yarn_layer_name, context, yarn_package_name)?,
        requirement,
    )
    .map_err(Into::into)
}

pub(crate) fn install_yarn(
    context: &BuildpackBuildContext,
    env: &mut Env,
    yarn_packument: &PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    // Note: yarn layer path is returned but not used for cleanup registration
    utils::npm_registry::install_package_layer(
        layer_name!("yarn"),
        context,
        env,
        yarn_packument,
        node_version,
    )?;
    Ok(())
}

#[instrument(skip_all)]
pub(crate) fn read_yarnrc(app_dir: &Path) -> Option<BuildpackResult<Yarnrc>> {
    let yarnrc_path = app_dir.join(Yarnrc::file_name());

    let contents = match std::fs::read_to_string(&yarnrc_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            return Some(Err(create_yarnrc_yml_read_error_message(&error).into()));
        }
    };
    tracing::info!({ YARNRC_CONTENTS } = contents, ".yarnrc.yml");

    match yaml_rust2::YamlLoader::load_from_str(&contents) {
        Ok(docs) if docs.len() == 1 => Some(Ok(Yarnrc(docs.into_iter().next()))),
        Ok(docs) if docs.is_empty() => Some(Ok(Yarnrc(None))),
        Ok(_) => Some(Err(
            create_yarnrc_yml_multiple_documents_error_message().into()
        )),
        Err(error) => Some(Err(create_yarnrc_yml_parse_error_message(&error).into())),
    }
}

#[derive(Debug)]
pub(crate) struct Yarnrc(Option<yaml_rust2::Yaml>);

impl Yarnrc {
    pub(crate) fn file_name() -> PathBuf {
        PathBuf::from(".yarnrc.yml")
    }

    pub(crate) fn yarn_path(&self) -> Option<PathBuf> {
        self.0
            .iter()
            .next()
            .and_then(|doc| doc["yarnPath"].as_str().map(PathBuf::from))
    }
}

fn create_yarnrc_yml_read_error_message(error: &std::io::Error) -> ErrorMessage {
    let yamlrc_yml = file_value(Yarnrc::file_name());
    error_message()
        .id("package_manager/yarn/read_yarnrc_yml")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error reading {yamlrc_yml}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {yamlrc_yml} to determine Yarn configuration but \
            the file can't be read.

            Suggestions:
            - Ensure the file has read permissions.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_yarnrc_yml_parse_error_message(error: &yaml_rust2::ScanError) -> ErrorMessage {
    let yamlrc_yml = file_value(Yarnrc::file_name());
    let yaml_spec_url = style::url("https://yaml.org/spec/1.2.2/");
    error_message()
        .id("package_manager/yarn/parse_yarnrc_yml")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error parsing {yamlrc_yml}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {yamlrc_yml} to determine Yarn configuration but \
            the file isn't valid YAML.

            Suggestions:
            - Ensure the file follows the YAML format described at {yaml_spec_url}
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_yarnrc_yml_multiple_documents_error_message() -> ErrorMessage {
    let yamlrc_yml = file_value(Yarnrc::file_name());
    let hyphens = style::value("---");
    error_message()
        .id("package_manager/yarn/multiple_documents_in_yarnrc_yml")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Multiple YAML documents found in {yamlrc_yml}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {yamlrc_yml} to determine Yarn configuration but \
            the file contains multiple YAML documents. There must only be a single document in this file.

            YAML documents are separated by a line contain three hyphens ({hyphens}).

            Suggestions:
            - Ensure the file has only a single document.
        " })
        .create()
}

pub(crate) fn link_vendored_yarn(
    context: &BuildpackBuildContext,
    env: &mut Env,
    vendored_yarn: &Path,
) -> BuildpackResult<()> {
    let bin_layer = context.uncached_layer(
        layer_name!("yarn_vendored"),
        UncachedLayerDefinition {
            build: true,
            launch: true,
        },
    )?;

    let bin_dir = bin_layer.path().join("bin");
    let yarn = bin_dir.join("yarn");
    let vendored_yarn = context.app_dir.join(vendored_yarn);

    std::fs::create_dir_all(&bin_dir)
        .and_then(|()| {
            std::fs::write(
                &yarn,
                formatdoc! { "
                    #!/usr/bin/env node
                    require({});
                ", serde_json::json!(vendored_yarn) },
            )
        })
        .and_then(|()| std::fs::metadata(&yarn))
        .and_then(|metadata| {
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(&yarn, permissions)
        })
        .map_err(|e| create_write_vendored_yarn_executable_error(&vendored_yarn, &e))?;

    let layer_env = &bin_layer.read_env()?;

    env.clone_from(&layer_env.apply(Scope::Build, env));

    Ok(())
}

fn create_write_vendored_yarn_executable_error(
    vendored_yarn_path: &Path,
    error: &std::io::Error,
) -> ErrorMessage {
    let vendored_yarn_path = style::value(vendored_yarn_path.to_string_lossy());
    let yarn = style::value("yarn");
    error_message()
        .id("package_manager/yarn/write_vendored_yarn_executable")
        .error_type(ErrorType::Internal)
        .header("Failed to create vendored Yarn executable")
        .body(formatdoc! { "
            An unexpected error occurred while attempting to create a {yarn} executable for {vendored_yarn_path}.
        " })
        .debug_info(error.to_string())
        .create()
}

pub(crate) fn get_version(env: &Env) -> BuildpackResult<Version> {
    Command::new("yarn")
        .envs(env)
        .arg("--version")
        .named_output()
        .try_into()
        .map_err(|e| create_get_yarn_version_command_error(&e).into())
}

fn create_get_yarn_version_command_error(error: &VersionCommandError) -> ErrorMessage {
    match error {
        VersionCommandError::Command(e) => error_message()
            .id("package_manager/yarn/get_version")
            .error_type(ErrorType::Internal)
            .header("Failed to determine Yarn version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current Yarn version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        VersionCommandError::Parse(stdout, e) => error_message()
            .id("package_manager/yarn/parse_version")
            .error_type(ErrorType::Internal)
            .header("Failed to parse Yarn version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing Yarn version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

pub(crate) fn install_dependencies(
    context: &BuildpackBuildContext,
    env: &mut Env,
    version: &Version,
) -> BuildpackResult<()> {
    print::bullet("Setting up yarn dependency cache");
    let yarn_env = disable_global_cache(version);
    let cache_strategy = determine_cache_strategy(&yarn_env.apply(Scope::Build, env), version)?;
    let yarn_env = configure_cache_layers(context, version, &cache_strategy, yarn_env)?;
    *env = yarn_env.apply(Scope::Build, env);
    write_config_layer(context, yarn_env)?;

    print::bullet("Installing dependencies");
    run_install(context, env, version, &cache_strategy)
}

/// How packages are cached between builds.
enum CacheStrategy {
    /// The app commits its cache, so it's installed from as-is.
    ZeroInstall,
    /// Yarn 1 uses a cached layer as its cache folder.
    Classic,
    /// Yarn 2+ mirrors packages into a cached layer, and the cache folder is a fresh layer each
    /// build. In Plug'n'Play mode the cache folder is shipped at launch, so this lets archives that
    /// aren't needed at runtime be removed from it without being downloaded again on the next build.
    Mirrored { pnp: bool },
}

/// Yarn is configured with build env vars, which take precedence over any user configuration and
/// don't modify the app's Yarn config files.
fn disable_global_cache(version: &Version) -> LayerEnv {
    let yarn_env = LayerEnv::new();
    if version.major() >= 2 {
        // If enabled, the `cacheFolder` setting is ignored and packages are stored in the global
        // cache (`$HOME/.yarn/berry/cache` by default), which isn't persisted into the build cache
        // or the final image. Yarn 4.x enables the global cache by default.
        yarn_env.chainable_insert(
            Scope::Build,
            ModificationBehavior::Override,
            "YARN_ENABLE_GLOBAL_CACHE",
            "false",
        )
    } else {
        yarn_env
    }
}

fn determine_cache_strategy(env: &Env, version: &Version) -> BuildpackResult<CacheStrategy> {
    let yarn_cache = get_cache_folder_config(env, version)?;
    Ok(if is_yarn_zero_install_mode(&yarn_cache) {
        CacheStrategy::ZeroInstall
    } else if version.major() == 1 {
        CacheStrategy::Classic
    } else {
        let node_linker = get_node_linker_config(env, version)?;
        CacheStrategy::Mirrored {
            pnp: matches!(node_linker, Some(NodeLinker::Pnp)),
        }
    })
}

fn configure_cache_layers(
    context: &BuildpackBuildContext,
    version: &Version,
    cache_strategy: &CacheStrategy,
    yarn_env: LayerEnv,
) -> BuildpackResult<LayerEnv> {
    match cache_strategy {
        CacheStrategy::ZeroInstall => {
            print::sub_bullet("Yarn zero-install detected. Skipping dependency cache.");
            Ok(yarn_env)
        }
        CacheStrategy::Classic => {
            let cache_dir = create_cache_directory(context, version, layer_name!("yarn_cache"))?;
            Ok(yarn_env.chainable_insert(
                Scope::Build,
                ModificationBehavior::Override,
                "YARN_CACHE_FOLDER",
                &cache_dir,
            ))
        }
        CacheStrategy::Mirrored { pnp } => {
            let mirror_dir = create_cache_directory(context, version, layer_name!("yarn_mirror"))?;
            let cache_dir = context
                .uncached_layer(
                    layer_name!("yarn_cache"),
                    UncachedLayerDefinition {
                        build: true,
                        // In Plug'n'Play mode, Yarn resolves packages directly from the cache so it must be present at launch
                        launch: *pnp,
                    },
                )?
                .path();

            // Yarn also stores data other than the mirror in the global folder, which shouldn't
            // accumulate in the cached layer.
            context.register_cleanup(CleanupTask::YarnGlobalFolder(mirror_dir.clone()));

            if *pnp && *version >= Version::new(2, 4, 1) {
                context.register_cleanup(CleanupTask::YarnCache {
                    app_dir: context.app_dir.clone(),
                    cache_dir: cache_dir.clone(),
                });
            }

            Ok(yarn_env
                .chainable_insert(
                    Scope::Build,
                    ModificationBehavior::Override,
                    "YARN_ENABLE_MIRROR",
                    "true",
                )
                // Yarn stores the mirror in the `cache` folder of the global folder
                .chainable_insert(
                    Scope::Build,
                    ModificationBehavior::Override,
                    "YARN_GLOBAL_FOLDER",
                    &mirror_dir,
                )
                .chainable_insert(
                    Scope::Build,
                    ModificationBehavior::Override,
                    "YARN_CACHE_FOLDER",
                    &cache_dir,
                ))
        }
    }
}

fn write_config_layer(context: &BuildpackBuildContext, yarn_env: LayerEnv) -> BuildpackResult<()> {
    context
        .uncached_layer(
            layer_name!("yarn_config"),
            UncachedLayerDefinition {
                build: true,
                launch: false,
            },
        )?
        .write_env(yarn_env)?;
    Ok(())
}

fn run_install(
    context: &BuildpackBuildContext,
    env: &Env,
    version: &Version,
    cache_strategy: &CacheStrategy,
) -> BuildpackResult<()> {
    let mut yarn_install_command = Command::new("yarn");
    yarn_install_command.envs(env);
    yarn_install_command.envs(node_gyp_env());
    yarn_install_command.arg("install");
    if version.major() == 1 {
        yarn_install_command.args(["--production=false", "--frozen-lockfile"]);
    } else {
        yarn_install_command.args(["--immutable", "--inline-builds"]);
        if matches!(cache_strategy, CacheStrategy::ZeroInstall) {
            yarn_install_command.arg("--immutable-cache");
        }
    }

    print::sub_stream_cmd(yarn_install_command)
        .map_err(|e| create_yarn_install_command_error(&e))?;

    // Register app directory for cleanup of non-deterministic Makefiles generated
    // by node-gyp. Native modules may be compiled into node_modules/ (yarn v1 and
    // node-modules linker) or .yarn/unplugged/ (PnP linker).
    context.register_cleanup(CleanupTask::NodeGypMakefiles(
        NodeGypArtifactLocation::AppDir(context.app_dir.clone()),
    ));

    Ok(())
}

fn get_cache_folder_config(env: &Env, version: &Version) -> Result<PathBuf, ErrorMessage> {
    Command::new("yarn")
        .envs(env)
        .arg("config")
        .arg("get")
        .arg(match version.major() {
            1 => "cache-folder",
            _ => "cacheFolder",
        })
        .named_output()
        .map(|output| PathBuf::from(output.stdout_lossy().trim()))
        .map_err(|e| create_get_cache_folder_config_error(&e))
}

fn create_get_cache_folder_config_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .id("package_manager/yarn/get_cache_folder_config")
        .error_type(ErrorType::Internal)
        .header("Failed to read configured Yarn cache directory")
        .body(formatdoc! {"
            The Heroku Node.js buildpack was unable to read the configuration for the Yarn cache directory.
        "})
        .debug_info(error.to_string())
        .create()
}

// A yarn cache is populated if it exists and has non-hidden files, indicating a zero-install mode
// should be employed.
fn is_yarn_zero_install_mode(yarn_cache: &Path) -> bool {
    yarn_cache.read_dir().is_ok_and(|mut contents| {
        contents.any(|entry| entry.is_ok_and(|e| !e.file_name().to_string_lossy().starts_with('.')))
    })
}

fn get_node_linker_config(
    env: &Env,
    version: &Version,
) -> Result<Option<NodeLinker>, ErrorMessage> {
    if version.major() == 1 {
        Ok(None)
    } else {
        let output = Command::new("yarn")
            .envs(env)
            .args(["config", "get", "nodeLinker"])
            .named_output()
            .map_err(|e| create_get_node_linker_config_error(&e))?;
        let node_linker = output.stdout_lossy().trim().parse()?;
        Ok(Some(node_linker))
    }
}

fn create_get_node_linker_config_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .id("package_manager/yarn/get_node_linker_config")
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::No))
        .header("Failed to read Yarn's nodeLinker configuration")
        .body(formatdoc! { "
            An unexpected value was encountered when trying to read Yarn's nodeLinker configuration. This \
            configuration is read using the command {read_cmd}.

            Suggestions:
            - Ensure the above command runs locally.
        ", read_cmd = style::command(error.name()) })
        .debug_info(error.to_string())
        .create()
}

#[derive(Debug)]
pub(crate) enum NodeLinker {
    Pnp,
    Pnpm,
    NodeModules,
}

impl FromStr for NodeLinker {
    type Err = ErrorMessage;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pnp" => Ok(NodeLinker::Pnp),
            "node-modules" => Ok(NodeLinker::NodeModules),
            "pnpm" => Ok(NodeLinker::Pnpm),
            _ => Err(create_unknown_node_linker_error(value)),
        }
    }
}

fn create_unknown_node_linker_error(value: &str) -> ErrorMessage {
    error_message()
        .id("package_manager/yarn/unknown_node_linker")
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header("Failed to parse Yarn's nodeLinker configuration")
        .body(formatdoc! { "
                An unexpected value was encountered when trying to read Yarn's nodeLinker configuration.
                Expected - 'pnp', 'node-modules', or 'pnpm'
                Actual   - '{value}'

                Suggestions:
                - Run {check_cmd} locally to check your 'nodeLinker' configuration.
                - Set an explicit 'nodeLinker' configuration in {yarnrc_yml} ({install_modes_url})
            ",
            check_cmd = style::command("yarn config get nodeLinker"),
            yarnrc_yml = style::value(".yarnrc.yml"),
            install_modes_url = style::url("https://yarnpkg.com/features/linkers")
        })
        .create()
}

fn create_cache_directory(
    context: &BuildpackBuildContext,
    version: &Version,
    name: LayerName,
) -> BuildpackResult<PathBuf> {
    let yarn_major_version = version.major().to_string();

    let yarn_cache_layer = context.cached_layer(
        name,
        CachedLayerDefinition {
            build: true,
            launch: false,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &YarnCacheDirectoryLayerMetadata, _| {
                restored_cache_directory_action(old_metadata, &yarn_major_version)
            },
        },
    )?;

    let previous_cache_usage_count = match yarn_cache_layer.state {
        LayerState::Restored {
            cause: previous_cache_usage_count,
        } => {
            print::sub_bullet("Restoring yarn dependency cache");
            Some(previous_cache_usage_count)
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                print::sub_bullet("Clearing yarn dependency cache");
            }
            None
        }
    };

    yarn_cache_layer.write_metadata(YarnCacheDirectoryLayerMetadata {
        cache_usage_count: next_cache_usage_count(previous_cache_usage_count),
        yarn_major_version,
        layer_version: YARN_CACHE_DIRECTORY_LAYER_VERSION.to_string(),
    })?;

    Ok(yarn_cache_layer.path())
}

/// Keeps a restored cache directory until it has been used for the maximum number of builds, so
/// packages that are no longer needed don't accumulate in it forever. Returns the previous usage
/// count alongside the action.
fn restored_cache_directory_action(
    old_metadata: &YarnCacheDirectoryLayerMetadata,
    yarn_major_version: &str,
) -> (RestoredLayerAction, f32) {
    let is_reusable = old_metadata.yarn_major_version == yarn_major_version
        && old_metadata.layer_version == YARN_CACHE_DIRECTORY_LAYER_VERSION
        && old_metadata.cache_usage_count < YARN_CACHE_DIRECTORY_MAX_CACHE_USAGE_COUNT;
    let action = if is_reusable {
        RestoredLayerAction::KeepLayer
    } else {
        RestoredLayerAction::DeleteLayer
    };
    (action, old_metadata.cache_usage_count)
}

/// Increments the usage count of a restored cache directory, or starts a new one at 1.
fn next_cache_usage_count(previous_cache_usage_count: Option<f32>) -> f32 {
    previous_cache_usage_count.map_or(1.0, |count| count + 1.0)
}

const YARN_CACHE_DIRECTORY_MAX_CACHE_USAGE_COUNT: f32 = 150.0;
const YARN_CACHE_DIRECTORY_LAYER_VERSION: &str = "1";

#[derive(Deserialize, Serialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
struct YarnCacheDirectoryLayerMetadata {
    // Usings float here due to [an issue with lifecycle's handling of integers](https://github.com/buildpacks/lifecycle/issues/884)
    cache_usage_count: f32,
    yarn_major_version: String,
    layer_version: String,
}

fn create_yarn_install_command_error(error: &fun_run::CmdError) -> ErrorMessage {
    let yarn_install = style::value(error.name());
    error_message()
        .id("package_manager/yarn/install")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::Yes,
        ))
        .header("Failed to install Node modules")
        .body(formatdoc! { "
            The Heroku Node.js buildpack uses the command {yarn_install} to install your Node modules. This command \
            failed and the buildpack cannot continue. This error can occur due to an unstable network connection. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
            - Check the status of the upstream Node module repository service at https://status.npmjs.org/
        " })
        .debug_info(error.to_string())
        .create()
}

pub(crate) fn run_script(name: impl AsRef<str>, env: &Env) -> Command {
    let mut command = Command::new("yarn");
    command.args(["run", name.as_ref()]);
    command.envs(env);
    command
}

pub(crate) fn prune_dev_dependencies(
    env: &Env,
    version: &Version,
    on_prune_command_error: impl FnOnce(&fun_run::CmdError) -> ErrorMessage,
) -> Result<(), ErrorMessage> {
    let mut prune_command = Command::new("yarn");
    prune_command.envs(env);
    if version.major() == 1 {
        prune_command.args([
            "install",
            "--production",
            "--frozen-lockfile",
            "--ignore-engines",
            "--ignore-scripts",
            "--prefer-offline",
        ])
    } else {
        let yarn_prune_plugin = install_yarn_prune_plugin()?;
        prune_command
            .env("YARN_PLUGINS", yarn_prune_plugin)
            .args(["heroku", "prune"])
    };
    print::sub_stream_cmd(prune_command)
        .map(|_| ())
        .map_err(|e| on_prune_command_error(&e))
}

/// Reads the Plug'n'Play data Yarn writes to the runtime file, and to `.pnp.data.json` when
/// `pnpEnableInlining` is disabled. Yarn 3+ always writes `.pnp.cjs`, but doesn't remove a
/// `.pnp.js` left over from Yarn 2, so `.pnp.cjs` is read when both exist. Yarn 2 writes
/// `.pnp.js`, or `.pnp.cjs` if the project's `type` is `module`, and removes the other one.
pub(crate) fn read_pnp_data(app_dir: &Path) -> std::io::Result<String> {
    let runtime = [".pnp.cjs", ".pnp.js"]
        .map(|name| app_dir.join(name))
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| app_dir.join(".pnp.cjs"));
    let mut pnp_data = fs_err::read_to_string(runtime)?;
    let data_file = app_dir.join(".pnp.data.json");
    if data_file.exists() {
        pnp_data.push_str(&fs_err::read_to_string(data_file)?);
    }
    Ok(pnp_data)
}

static PACKAGE_LOCATION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""packageLocation":\s*"([^"]*)""#).expect("Package location regex should be valid")
});

/// Lists the cache archives referenced by the package locations in the Plug'n'Play data, such as
/// `../yarn_cache/debug-npm-4.3.4-4513954577-cedbec4529.zip/node_modules/debug/`.
///
/// Yarn doesn't document this format, so the result is only returned if it looks trustworthy.
/// Listing too many archives is harmless, but listing too few would remove packages the app needs.
pub(crate) fn get_pnp_package_archives(
    pnp_data: &str,
    cache_dir: &Path,
) -> Result<HashSet<String>, String> {
    let locations = PACKAGE_LOCATION
        .captures_iter(pnp_data)
        .map(|captures| captures[1].to_string())
        .collect::<Vec<_>>();
    if !locations.iter().any(|location| location == "./") {
        return Err("the project's package location is missing from the Plug'n'Play data".into());
    }
    let archives = locations
        .iter()
        .flat_map(|location| location.split('/'))
        .filter(|segment| {
            Path::new(segment)
                .extension()
                .is_some_and(|ext| ext == "zip")
        })
        .map(String::from)
        .collect::<HashSet<_>>();
    if let Some(archive) = archives
        .iter()
        .find(|archive| !cache_dir.join(archive).is_file())
    {
        return Err(format!("{archive} isn't in the Yarn cache"));
    }
    Ok(archives)
}

const YARN_PRUNE_PLUGIN_SOURCE: &str = include_str!("@yarnpkg/plugin-prune-dev-dependencies.js");

fn install_yarn_prune_plugin() -> Result<PathBuf, ErrorMessage> {
    let temp_dir = tempfile::tempdir()
        .map(tempfile::TempDir::keep)
        .map_err(|e| create_yarn_install_prune_plugin_error(&e))?;

    let plugin_source = temp_dir.join("plugin-prune-dev-dependencies.js");
    std::fs::write(&plugin_source, YARN_PRUNE_PLUGIN_SOURCE)
        .map_err(|e| create_yarn_install_prune_plugin_error(&e))?;

    Ok(plugin_source)
}

fn create_yarn_install_prune_plugin_error(error: &std::io::Error) -> ErrorMessage {
    error_message()
        .id("package_manager/yarn/install_prune_plugin")
        .error_type(ErrorType::Internal)
        .header("Failed to install Yarn plugin for pruning")
        .body(formatdoc! { "
            The Heroku Node.js buildpack uses a custom plugin for Yarn to handle pruning \
            of dev dependencies. An unexpected error was encountered while trying to install it.
        " })
        .debug_info(error.to_string())
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_cmd_error};

    #[test]
    fn version_parse_error() {
        assert_error_snapshot(&create_get_yarn_version_command_error(
            &VersionCommandError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    #[test]
    fn version_command_error() {
        assert_error_snapshot(&create_get_yarn_version_command_error(
            &VersionCommandError::Command(create_cmd_error("yarn --version")),
        ));
    }

    #[test]
    fn read_yarnrc_success() {
        let app_dir = tempfile::tempdir().unwrap();
        let yarnrc_path = app_dir.path().join(".yarnrc.yml");

        // no yarnrc
        let yarnrc = read_yarnrc(app_dir.path());
        assert!(yarnrc.is_none());

        // empty yarnrc
        std::fs::write(&yarnrc_path, "").unwrap();
        let yarnrc = read_yarnrc(app_dir.path()).unwrap().unwrap();
        assert_eq!(yarnrc.yarn_path(), None);

        // yarnrc with yarnPath
        std::fs::write(&yarnrc_path, "yarnPath: /path/to/yarn").unwrap();
        let yarnrc = read_yarnrc(app_dir.path()).unwrap().unwrap();
        assert_eq!(yarnrc.yarn_path(), Some("/path/to/yarn".into()));
    }

    #[test]
    fn yarnrc_multiple_docs_error() {
        let app_dir = tempfile::tempdir().unwrap();
        let yarnrc_path = app_dir.path().join(".yarnrc.yml");
        std::fs::write(
            &yarnrc_path,
            "---\nyarnPath: /path/to/yarn\n---\nyarnPath: /path/to/yarn",
        )
        .unwrap();
        let error = read_yarnrc(app_dir.path()).unwrap().unwrap_err();
        match error {
            crate::BuildpackError::BuildpackError(message) => {
                assert_error_snapshot(&message);
            }
            _ => panic!("Not the expected error type"),
        }
    }

    #[test]
    fn yarnrc_parse_error() {
        let app_dir = tempfile::tempdir().unwrap();
        let yarnrc_path = app_dir.path().join(".yarnrc.yml");
        std::fs::write(&yarnrc_path, "---\nyarnPath: \"").unwrap();
        let error = read_yarnrc(app_dir.path()).unwrap().unwrap_err();
        match error {
            crate::BuildpackError::BuildpackError(message) => {
                assert_error_snapshot(&message);
            }
            _ => panic!("Not the expected error type"),
        }
    }

    #[test]
    fn yarnrc_read_error() {
        let app_dir = tempfile::tempdir().unwrap();
        let yarnrc_path = app_dir.path().join(".yarnrc.yml");
        std::fs::create_dir(yarnrc_path).unwrap();
        let error = read_yarnrc(app_dir.path()).unwrap().unwrap_err();
        match error {
            crate::BuildpackError::BuildpackError(message) => {
                assert_error_snapshot(&message);
            }
            _ => panic!("Not the expected error type"),
        }
    }

    #[test]
    fn write_vendored_yarn_executable_error() {
        assert_error_snapshot(&create_write_vendored_yarn_executable_error(
            &PathBuf::from("/path/to/yarn"),
            &std::io::Error::other("Not found"),
        ));
    }

    // Archives that a fixture's Plug'n'Play data doesn't reference, added to the cache folder to
    // check that they aren't listed or kept: dev dependencies, unplugged packages, and the original
    // of a package patched by Yarn.
    const UNUSED_ARCHIVES: [&str; 6] = [
        "depd-npm-2.0.0-b6c51a4b43-58bd06ec20.zip",
        "dotenv-npm-16.4.5-bcb20eb95d-48d9287007.zip",
        "is-number-npm-7.0.0-060086935c-b4686d0d30.zip",
        "local-pkg-file-448fc06f1b-3f47fc80e1.zip",
        "picocolors-npm-1.1.1-4fede47cf1-e2e3e8170a.zip",
        "resolve-npm-1.22.8-098f379dfe-07e179f437.zip",
    ];

    fn pnp_data_fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/yarn-data")
            .join(name)
    }

    /// The archives a fixture references after pruning dev dependencies, as listed by Yarn's
    /// Plug'n'Play API when the fixture was generated. See `tests/fixtures/yarn-data/README.md`.
    fn expected_archives(fixture: &str) -> Vec<String> {
        std::fs::read_to_string(pnp_data_fixture(fixture).join("expected-archives.txt"))
            .unwrap()
            .lines()
            .map(String::from)
            .collect()
    }

    fn create_cache_dir<S: AsRef<str>>(files: &[S]) -> tempfile::TempDir {
        let cache_dir = tempfile::tempdir().unwrap();
        for file in files {
            std::fs::write(cache_dir.path().join(file.as_ref()), "").unwrap();
        }
        cache_dir
    }

    fn with_unused_archives(archives: &[String]) -> Vec<String> {
        archives
            .iter()
            .cloned()
            .chain(UNUSED_ARCHIVES.map(String::from))
            .collect()
    }

    fn assert_pnp_package_archives(fixture: &str) {
        let expected = expected_archives(fixture);
        let cache_dir = create_cache_dir(&with_unused_archives(&expected));
        let pnp_data = read_pnp_data(&pnp_data_fixture(fixture)).unwrap();
        assert_eq!(
            get_pnp_package_archives(&pnp_data, cache_dir.path()).unwrap(),
            expected.into_iter().collect()
        );
    }

    #[test]
    fn get_pnp_package_archives_yarn_2() {
        assert_pnp_package_archives("yarn-2.4.1");
    }

    #[test]
    fn get_pnp_package_archives_yarn_3() {
        assert_pnp_package_archives("yarn-3.8.7");
    }

    #[test]
    fn get_pnp_package_archives_yarn_4() {
        assert_pnp_package_archives("yarn-4.19.0");
    }

    #[test]
    fn get_pnp_package_archives_with_inlining_disabled() {
        assert_pnp_package_archives("yarn-4.19.0-no-inlining");
    }

    #[test]
    fn get_pnp_package_archives_with_only_unplugged_packages() {
        assert_pnp_package_archives("yarn-4.19.0-unplugged-only");
    }

    #[test]
    fn get_pnp_package_archives_with_no_dependencies() {
        assert_pnp_package_archives("yarn-4.19.0-no-dependencies");
    }

    #[test]
    fn get_pnp_package_archives_without_project_location() {
        let cache_dir = create_cache_dir(&expected_archives("yarn-4.19.0"));
        let pnp_data = r#""packageLocation": "../yarn_cache/debug-npm-4.3.4-4513954577-cedbec4529.zip/node_modules/debug/""#;
        assert_eq!(
            get_pnp_package_archives(pnp_data, cache_dir.path()).unwrap_err(),
            "the project's package location is missing from the Plug'n'Play data"
        );
    }

    #[test]
    fn get_pnp_package_archives_with_archive_missing_from_cache() {
        let archives = expected_archives("yarn-4.19.0");
        let cache_dir = create_cache_dir(&archives[1..]);
        let pnp_data = read_pnp_data(&pnp_data_fixture("yarn-4.19.0")).unwrap();
        assert_eq!(
            get_pnp_package_archives(&pnp_data, cache_dir.path()).unwrap_err(),
            format!("{} isn't in the Yarn cache", archives[0])
        );
    }

    #[test]
    fn read_pnp_data_prefers_pnp_cjs_over_leftover_pnp_js() {
        let app_dir = tempfile::tempdir().unwrap();
        std::fs::write(app_dir.path().join(".pnp.cjs"), "current").unwrap();
        std::fs::write(app_dir.path().join(".pnp.js"), "leftover").unwrap();
        assert_eq!(read_pnp_data(app_dir.path()).unwrap(), "current");
    }

    #[test]
    fn read_pnp_data_without_pnp_runtime() {
        let app_dir = tempfile::tempdir().unwrap();
        assert_eq!(
            read_pnp_data(app_dir.path()).unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
    }

    #[test]
    fn get_cache_folder_config_error() {
        assert_error_snapshot(&create_get_cache_folder_config_error(&create_cmd_error(
            "yarn config get cacheFolder",
        )));
    }

    #[test]
    fn get_node_linker_config_error() {
        assert_error_snapshot(&create_get_node_linker_config_error(&create_cmd_error(
            "yarn config get nodeLinker",
        )));
    }

    #[test]
    fn unknown_node_linker_error() {
        assert_error_snapshot(&create_unknown_node_linker_error("unknown"));
    }

    #[test]
    fn yarn_install_command_error() {
        assert_error_snapshot(&create_yarn_install_command_error(&create_cmd_error(
            "yarn install",
        )));
    }

    #[test]
    fn test_is_zero_install_mode_when_cache_is_populated() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("./tests/fixtures/yarn-3-modules-zero/.yarn/cache");
        assert!(
            is_yarn_zero_install_mode(&path),
            "Expected zero-install app to have a populated cache"
        );
    }

    #[test]
    fn test_is_zero_install_mode_when_cache_is_unpopulated() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("./tests/fixtures/yarn-4-pnp-nonzero/.yarn/cache");
        assert!(
            !is_yarn_zero_install_mode(&path),
            "Expected non-zero-install app to have an unpopulated cache"
        );
    }

    #[test]
    fn yarn_install_prune_plugin_error() {
        assert_error_snapshot(&create_yarn_install_prune_plugin_error(
            &std::io::Error::other("Out of disk space"),
        ));
    }

    fn cache_directory_metadata(
        cache_usage_count: f32,
        yarn_major_version: &str,
        layer_version: &str,
    ) -> YarnCacheDirectoryLayerMetadata {
        YarnCacheDirectoryLayerMetadata {
            cache_usage_count,
            yarn_major_version: yarn_major_version.to_string(),
            layer_version: layer_version.to_string(),
        }
    }

    #[test]
    fn restored_cache_directory_is_kept_below_max_usage_count() {
        for cache_usage_count in [1.0, 2.0, 149.0] {
            let (action, previous_cache_usage_count) = restored_cache_directory_action(
                &cache_directory_metadata(cache_usage_count, "4", "1"),
                "4",
            );
            assert!(matches!(action, RestoredLayerAction::KeepLayer));
            assert_eq!(previous_cache_usage_count, cache_usage_count);
        }
    }

    #[test]
    fn restored_cache_directory_is_deleted_at_max_usage_count() {
        let (action, _) =
            restored_cache_directory_action(&cache_directory_metadata(150.0, "4", "1"), "4");
        assert!(matches!(action, RestoredLayerAction::DeleteLayer));
    }

    #[test]
    fn restored_cache_directory_is_deleted_when_yarn_major_version_changes() {
        let (action, _) =
            restored_cache_directory_action(&cache_directory_metadata(1.0, "3", "1"), "4");
        assert!(matches!(action, RestoredLayerAction::DeleteLayer));
    }

    #[test]
    fn restored_cache_directory_is_deleted_when_layer_version_changes() {
        let (action, _) =
            restored_cache_directory_action(&cache_directory_metadata(1.0, "4", "0"), "4");
        assert!(matches!(action, RestoredLayerAction::DeleteLayer));
    }

    #[test]
    fn next_cache_usage_count_increments_restored_count() {
        assert_eq!(next_cache_usage_count(Some(1.0)), 2.0);
    }

    #[test]
    fn next_cache_usage_count_resets_for_new_cache_directory() {
        assert_eq!(next_cache_usage_count(None), 1.0);
    }
}
