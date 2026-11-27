use crate::cleanup::{CleanupTask, NodeGypArtifactLocation, Omit};
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
    let packument = if requires_yarn_6_or_later(requirement) {
        let package_name =
            yarn_native_binary_package_name(&context.target.os, &context.target.arch)?;
        packument_layer(layer_name!("yarn_native_packument"), context, &package_name)?
    } else if requirement.allows_any(&YARN_BERRY_RANGE) {
        packument_layer(
            layer_name!("yarnpkg_cli-dist_packument"),
            context,
            "@yarnpkg/cli-dist",
        )?
    } else {
        packument_layer(layer_name!("yarn_packument"), context, "yarn")?
    };
    resolve_package_packument(&packument, requirement).map_err(Into::into)
}

/// Yarn 6+ is only selected when the requirement can't be satisfied by an earlier major (e.g.; an
/// exact `packageManager` pin) since it isn't a drop-in replacement for Yarn 2-4 projects.
pub(crate) fn requires_yarn_6_or_later(requirement: &VersionRange) -> bool {
    requirement
        .min_version()
        .is_some_and(|version| version.major() >= 6)
}

/// Yarn 6+ is a native binary that is only published as platform-specific packages. The binaries
/// are statically linked against musl so they run on Heroku's glibc-based images.
fn yarn_native_binary_package_name(os: &str, arch: &str) -> BuildpackResult<String> {
    match (os, arch) {
        ("linux", "amd64") => Ok("@yarnpkg/yarn-x86_64-unknown-linux-musl".to_string()),
        ("linux", "arm64") => Ok("@yarnpkg/yarn-aarch64-unknown-linux-musl".to_string()),
        _ => Err(create_yarn_unsupported_target_error(os, arch).into()),
    }
}

fn create_yarn_unsupported_target_error(os: &str, arch: &str) -> ErrorMessage {
    let target = style::value(format!("{os}-{arch}"));
    error_message()
        .id("package_manager/yarn/unsupported_native_binary_target")
        .error_type(ErrorType::Internal)
        .header("Unsupported Yarn target")
        .body(formatdoc! { "
            Yarn 6 and later ship their native binary as a platform-specific package, but the \
            buildpack does not know which package to use for the current build target ({target}).
        " })
        .create()
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
    if version.major() >= 6
        && let Some(Ok(yarnrc)) = read_yarnrc(&context.app_dir)
        && let Some(yarn_path) = yarnrc.yarn_path()
    {
        print::warning(formatdoc! { "
            Warning: Ignoring {yarn_path} in {yarnrc}

            Yarn 6 no longer supports the {yarn_path_setting} setting and uses the version declared \
            in the {package_manager} field of {package_json} instead. Remove {yarn_path_setting} from \
            {yarnrc} and delete the vendored Yarn release from your project.
        ",
            yarn_path = style::value(yarn_path.to_string_lossy()),
            yarn_path_setting = style::value("yarnPath"),
            yarnrc = file_value(Yarnrc::file_name()),
            package_manager = style::value("packageManager"),
            package_json = file_value("package.json"),
        });
    }

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
    ZeroInstall { cache_dir: PathBuf, pnp: bool },
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
    let is_pnp = || {
        get_node_linker_config(env, version)
            .map(|node_linker| matches!(node_linker, Some(NodeLinker::Pnp)))
    };
    Ok(if is_yarn_zero_install_mode(&yarn_cache) {
        CacheStrategy::ZeroInstall {
            pnp: is_pnp()?,
            cache_dir: yarn_cache,
        }
    } else if version.major() == 1 {
        CacheStrategy::Classic
    } else {
        CacheStrategy::Mirrored { pnp: is_pnp()? }
    })
}

fn configure_cache_layers(
    context: &BuildpackBuildContext,
    version: &Version,
    cache_strategy: &CacheStrategy,
    yarn_env: LayerEnv,
) -> BuildpackResult<LayerEnv> {
    match cache_strategy {
        CacheStrategy::ZeroInstall { cache_dir, pnp } => {
            print::sub_bullet("Yarn zero-install detected. Skipping dependency cache.");
            // The committed cache holds every package archive, including dev dependencies, and is
            // shipped at launch with the app. Only archives in the app are removed.
            if version.major() >= 2 && cache_dir.starts_with(&context.app_dir) {
                context.register_cleanup(CleanupTask::YarnCache {
                    app_dir: context.app_dir.clone(),
                    cache_dir: cache_dir.clone(),
                    mirror_cache_dir: None,
                    // The archives Plug'n'Play uses can't be determined before Yarn 2.4.1
                    omit: if !*pnp {
                        None
                    } else if *version >= Version::new(2, 4, 1) {
                        Some(Omit::ArchivesUsedByPnp)
                    } else {
                        Some(Omit::CacheDirContents)
                    },
                });
            }
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

            context.register_cleanup(CleanupTask::YarnCache {
                app_dir: context.app_dir.clone(),
                cache_dir: cache_dir.clone(),
                // Yarn stores the mirror in the `cache` folder of the global folder
                mirror_cache_dir: Some(mirror_dir.join("cache")),
                // The cache folder is only shipped at launch for Plug'n'Play, and the archives it
                // uses can't be determined before Yarn 2.4.1
                omit: Some(if *pnp && *version >= Version::new(2, 4, 1) {
                    Omit::ArchivesUsedByPnp
                } else {
                    Omit::CacheDirContents
                }),
            });

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

pub(crate) fn list_package_archives(folder: &Path) -> std::io::Result<Vec<String>> {
    let mut archives = vec![];
    for entry in fs_err::read_dir(folder)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "zip")
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            archives.push(name.to_string());
        }
    }
    Ok(archives)
}

/// Archives are named `<slug>-<suffix>.zip` in both the mirror and the cache folder. The slug
/// identifies the package (name, protocol, version, and locator hash) and is what Yarn uses to find
/// an archive in the mirror. The suffix is the cache key in the mirror and the cache key or checksum
/// in the cache folder, so only the slug is compared.
///
/// When the cache key changes (e.g. a Yarn upgrade or a different `compressionLevel`), Yarn writes a
/// new mirror archive for the same slug and leaves the old one behind. The mirror archive Yarn uses
/// is the one with the lockfile's cache key, since the install writes the cache key from the same
/// Yarn config to the lockfile (or fails, if it's immutable and Yarn won't update the cache key).
/// Without a cache key, every archive for a used slug is kept.
pub(crate) fn get_stale_mirror_archives<'a>(
    cache_archives: &[String],
    mirror_archives: &'a [String],
    cache_key: Option<&str>,
) -> Vec<&'a str> {
    let used_slugs: HashSet<&str> = cache_archives
        .iter()
        .filter_map(|archive| get_package_archive_slug(archive))
        .collect();
    mirror_archives
        .iter()
        .filter(|archive| {
            archive
                .strip_suffix(".zip")
                .and_then(|archive| archive.rsplit_once('-'))
                .is_some_and(|(slug, suffix)| {
                    !used_slugs.contains(slug) || cache_key.is_some_and(|key| suffix != key)
                })
        })
        .map(String::as_str)
        .collect()
}

/// Reads `cacheKey` from the lockfile's `__metadata` entry.
pub(crate) fn get_lockfile_cache_key(lockfile: &str) -> Option<String> {
    let docs = yaml_rust2::YamlLoader::load_from_str(lockfile).ok()?;
    match &docs.first()?["__metadata"]["cacheKey"] {
        yaml_rust2::Yaml::String(cache_key) => Some(cache_key.clone()),
        // Cache keys without a compression level, like `8`, are parsed as integers
        yaml_rust2::Yaml::Integer(cache_key) => Some(cache_key.to_string()),
        _ => None,
    }
}

fn get_package_archive_slug(archive: &str) -> Option<&str> {
    archive
        .strip_suffix(".zip")?
        .rsplit_once('-')
        .map(|(slug, _)| slug)
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
        if matches!(cache_strategy, CacheStrategy::ZeroInstall { .. }) {
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
    } else if version.major() >= 4 {
        // Yarn's replacement for Yarn 1's `yarn install --production`, see:
        // - https://yarnpkg.com/migration/guide#cli-changes
        // - https://yarnpkg.com/cli/workspaces/focus
        prune_command.args(["workspaces", "focus", "--all", "--production"])
    } else {
        // Yarn 2 and 3 don't bundle the workspace-tools plugin that provides `yarn workspaces focus`
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
    fn requires_yarn_6_or_later_only_for_yarn_6_requirements() {
        for requirement in ["6.0.0-rc.23", "6.0.0", "6.x", "^6.1.0", ">=6"] {
            assert!(
                requires_yarn_6_or_later(&VersionRange::parse(requirement).unwrap()),
                "expected {requirement} to require Yarn 6+"
            );
        }
        for requirement in ["1.22.x", "4.0.0", ">=1.22.19", ">=4", "*", "latest"] {
            assert!(
                !requires_yarn_6_or_later(&VersionRange::parse(requirement).unwrap()),
                "expected {requirement} not to require Yarn 6+"
            );
        }
    }

    #[test]
    fn yarn_native_binary_package_name_for_supported_targets() {
        assert_eq!(
            yarn_native_binary_package_name("linux", "amd64").unwrap(),
            "@yarnpkg/yarn-x86_64-unknown-linux-musl"
        );
        assert_eq!(
            yarn_native_binary_package_name("linux", "arm64").unwrap(),
            "@yarnpkg/yarn-aarch64-unknown-linux-musl"
        );
        assert!(yarn_native_binary_package_name("darwin", "arm64").is_err());
    }

    #[test]
    fn unsupported_target_error() {
        assert_error_snapshot(&create_yarn_unsupported_target_error("darwin", "arm64"));
    }

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

    fn archives(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    fn sorted_stale_mirror_archives<'a>(
        cache_archives: &[String],
        mirror_archives: &'a [String],
        cache_key: Option<&str>,
    ) -> Vec<&'a str> {
        let mut stale = get_stale_mirror_archives(cache_archives, mirror_archives, cache_key);
        stale.sort_unstable();
        stale
    }

    #[test]
    fn stale_mirror_archives_are_matched_by_slug() {
        // Yarn 4 names cache folder archives with the checksum and mirror archives with the cache key
        let cache_archives = archives(&[
            "@types-node-npm-20.0.0-1234567890-abcdef0123.zip",
            "express-npm-4.21.2-cb2e2a2f28-38168fd0a3.zip",
            "resolve-patch-4254c24959-0b0f0e7a2a.zip",
        ]);
        let mirror_archives = archives(&[
            "@types-node-npm-20.0.0-1234567890-10c0.zip",
            "dotenv-npm-16.6.1-01334288ea-10c0.zip",
            "express-npm-4.21.2-cb2e2a2f28-10c0.zip",
            "express-npm-4.21.1-5a3d1e8b0c-10c0.zip",
            "resolve-patch-4254c24959-10c0.zip",
        ]);
        assert_eq!(
            sorted_stale_mirror_archives(&cache_archives, &mirror_archives, Some("10c0")),
            [
                "dotenv-npm-16.6.1-01334288ea-10c0.zip",
                "express-npm-4.21.1-5a3d1e8b0c-10c0.zip"
            ]
        );
    }

    #[test]
    fn stale_mirror_archives_with_cache_key_suffixes() {
        // Archives without a checksum use the cache key in the cache folder too (e.g. Yarn 2 and 3)
        let cache_archives = archives(&["accepts-npm-1.3.8-9a812371c9-8.zip"]);
        let mirror_archives = archives(&[
            "accepts-npm-1.3.8-9a812371c9-8.zip",
            "ms-npm-2.1.3-81ff3cfac1-8.zip",
        ]);
        assert_eq!(
            sorted_stale_mirror_archives(&cache_archives, &mirror_archives, Some("8")),
            ["ms-npm-2.1.3-81ff3cfac1-8.zip"]
        );
    }

    #[test]
    fn stale_mirror_archives_after_cache_key_change() {
        let cache_archives = archives(&["express-npm-4.21.2-cb2e2a2f28-38168fd0a3.zip"]);
        let mirror_archives = archives(&[
            "express-npm-4.21.2-cb2e2a2f28-10c0.zip",
            "express-npm-4.21.2-cb2e2a2f28-10c9.zip",
        ]);
        assert_eq!(
            sorted_stale_mirror_archives(&cache_archives, &mirror_archives, Some("10c9")),
            ["express-npm-4.21.2-cb2e2a2f28-10c0.zip"]
        );
    }

    #[test]
    fn stale_mirror_archives_without_a_cache_key() {
        let cache_archives = archives(&["express-npm-4.21.2-cb2e2a2f28-38168fd0a3.zip"]);
        let mirror_archives = archives(&[
            "dotenv-npm-16.6.1-01334288ea-10c0.zip",
            "express-npm-4.21.2-cb2e2a2f28-10c0.zip",
            "express-npm-4.21.2-cb2e2a2f28-10c9.zip",
        ]);
        assert_eq!(
            sorted_stale_mirror_archives(&cache_archives, &mirror_archives, None),
            ["dotenv-npm-16.6.1-01334288ea-10c0.zip"]
        );
    }

    #[test]
    fn stale_mirror_archives_ignores_unrecognized_files() {
        let mirror_archives = archives(&["notes.txt", "archive.zip"]);
        assert_eq!(
            sorted_stale_mirror_archives(&[], &mirror_archives, Some("10c0")),
            Vec::<&str>::new()
        );
    }

    #[test]
    fn lockfile_cache_key() {
        let lockfile = indoc::indoc! {r#"
            # This file is generated by running "yarn install" inside your project.
            # Manual changes might be lost - proceed with caution!

            __metadata:
              version: 8
              cacheKey: 10c0

            "express@npm:4.21.2":
              version: 4.21.2
        "#};
        assert_eq!(get_lockfile_cache_key(lockfile).as_deref(), Some("10c0"));
    }

    #[test]
    fn lockfile_cache_key_quoted() {
        let lockfile = "__metadata:\n  version: 4\n  cacheKey: \"7\"\n";
        assert_eq!(get_lockfile_cache_key(lockfile).as_deref(), Some("7"));
    }

    #[test]
    fn lockfile_cache_key_integer() {
        let lockfile = "__metadata:\n  version: 6\n  cacheKey: 8\n";
        assert_eq!(get_lockfile_cache_key(lockfile).as_deref(), Some("8"));
    }

    #[test]
    fn lockfile_cache_key_missing() {
        let lockfile = indoc::indoc! {r#"
            __metadata:
              version: 8

            "express@npm:4.21.2":
              cacheKey: 10c0
        "#};
        assert_eq!(get_lockfile_cache_key(lockfile), None);
    }

    /// See `tests/fixtures/yarn-data/README.md`.
    fn assert_mirror_fixture_stale_archives(fixture: &str) {
        let fixture_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/yarn-data")
            .join(fixture);
        let read_lines = |file: &str| -> Vec<String> {
            std::fs::read_to_string(fixture_dir.join(file))
                .unwrap()
                .lines()
                .map(String::from)
                .collect()
        };
        let lockfile = std::fs::read_to_string(fixture_dir.join("yarn.lock")).unwrap();
        let cache_key =
            get_lockfile_cache_key(&lockfile).expect("lockfile should have a cache key");
        assert_eq!(
            sorted_stale_mirror_archives(
                &read_lines("cache-archives.txt"),
                &read_lines("mirror-archives.txt"),
                Some(&cache_key)
            ),
            read_lines("expected-stale-archives.txt")
        );
    }

    #[test]
    fn stale_mirror_archives_yarn_2_dependencies_removed() {
        assert_mirror_fixture_stale_archives("yarn-2.4.1-dependencies-removed");
    }

    #[test]
    fn stale_mirror_archives_yarn_2_cache_key_changed() {
        assert_mirror_fixture_stale_archives("yarn-2.4.1-cache-key-changed");
    }

    #[test]
    fn stale_mirror_archives_yarn_3_dependencies_removed() {
        assert_mirror_fixture_stale_archives("yarn-3.8.7-dependencies-removed");
    }

    #[test]
    fn stale_mirror_archives_yarn_3_cache_key_changed() {
        assert_mirror_fixture_stale_archives("yarn-3.8.7-cache-key-changed");
    }

    #[test]
    fn stale_mirror_archives_yarn_4_dependencies_removed() {
        assert_mirror_fixture_stale_archives("yarn-4.19.0-dependencies-removed");
    }

    #[test]
    fn stale_mirror_archives_yarn_4_cache_key_changed() {
        assert_mirror_fixture_stale_archives("yarn-4.19.0-cache-key-changed");
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
