use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::npm_registry::{PackagePackument, packument_layer, resolve_package_packument};
use crate::utils::vrs::{Requirement, Version, VersionCommandError};
use crate::{BuildpackBuildContext, BuildpackResult, utils};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use indoc::formatdoc;
use libcnb::Env;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
    UncachedLayerDefinition,
};
use libcnb::layer_env::Scope;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::LazyLock;

static YARN_BERRY_RANGE: LazyLock<Requirement> = LazyLock::new(|| {
    Requirement::parse(">=2").expect("Yarn berry requirement range should be valid")
});

pub(crate) static DEFAULT_YARN_REQUIREMENT: LazyLock<Requirement> = LazyLock::new(|| {
    Requirement::parse("1.22.x").expect("Default Yarn requirement should be valid")
});

pub(crate) fn resolve_yarn_package_packument(
    context: &BuildpackBuildContext,
    requirement: &Requirement,
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
    utils::npm_registry::install_package_layer(
        layer_name!("yarn"),
        context,
        env,
        yarn_packument,
        node_version,
    )
    .map_err(Into::into)
}

pub(crate) fn read_yarnrc(app_dir: &Path) -> Option<BuildpackResult<Yarnrc>> {
    let yarnrc_path = app_dir.join(Yarnrc::file_name());

    let contents = match std::fs::read_to_string(&yarnrc_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            return Some(Err(create_yarnrc_yml_read_error_message(&error).into()));
        }
    };

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
    yarn_path: &Path,
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
    let full_yarn_path = context.app_dir.join(yarn_path);

    std::fs::create_dir_all(&bin_dir)
        .map_err(|e| create_write_vendored_yarn_link_error(yarn_path, &e))?;
    utils::fs::symlink_executable(full_yarn_path, yarn)
        .map_err(|e| create_write_vendored_yarn_link_error(yarn_path, &e))?;

    let layer_env = &bin_layer.read_env()?;

    env.clone_from(&layer_env.apply(Scope::Build, env));

    Ok(())
}

fn create_write_vendored_yarn_link_error(yarn_path: &Path, error: &std::io::Error) -> ErrorMessage {
    let yarn_path = style::value(yarn_path.to_string_lossy());
    let yarn = style::value("yarn");
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to create vendored Yarn link")
        .body(formatdoc! { "
            An unexpected error occurred while attempting to create a symbolic link from {yarn_path} to {yarn}.
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
            .error_type(ErrorType::Internal)
            .header("Failed to determine Yarn version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current Yarn version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        VersionCommandError::Parse(stdout, e) => error_message()
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
    env: &Env,
    version: &Version,
) -> BuildpackResult<()> {
    print::bullet("Setting up yarn dependency cache");
    ensure_global_cache_is_disabled(env, version)?;

    let yarn_cache = get_cache_folder_config(env, version)?;
    let zero_install_mode = is_yarn_zero_install_mode(&yarn_cache);
    if zero_install_mode {
        print::sub_bullet("Yarn zero-install detected. Skipping dependency cache.");
    } else {
        let node_linker = get_node_linker_config(env, version)?;
        let cache_dir = create_cache_directory(context, version, node_linker.as_ref())?;
        set_cache_folder_config(env, version, &cache_dir)?;
    }

    print::bullet("Installing dependencies");
    let mut yarn_install_command = Command::new("yarn");
    yarn_install_command.envs(env);
    yarn_install_command.arg("install");
    if version.major() == 1 {
        yarn_install_command.args(["--production=false", "--frozen-lockfile"]);
    } else {
        yarn_install_command.args(["--immutable", "--inline-builds"]);
        if zero_install_mode {
            yarn_install_command.arg("--immutable-cache");
        }
    }

    print::sub_stream_cmd(yarn_install_command)
        .map_err(|e| create_yarn_install_command_error(&e))?;

    Ok(())
}

fn ensure_global_cache_is_disabled(env: &Env, version: &Version) -> Result<(), ErrorMessage> {
    // Execute `yarn config set enableGlobalCache false`. This setting is
    // only available on yarn >= 2. If set to `true`, the `cacheFolder` setting
    // will be ignored, and cached dependencies will be stored in the global
    // Yarn cache (`$HOME/.yarn/berry/cache` by default), which isn't
    // persisted into the build cache or the final image. Yarn 2.x and 3.x have
    // a default value to `false`. Yarn 4.x has a default value of `true`.
    if version.major() >= 2 {
        print::sub_stream_cmd(
            Command::new("yarn")
                .args(["config", "set", "enableGlobalCache", "false"])
                .envs(env),
        )
        .map_err(|e| create_ensure_global_cache_is_disabled_error(&e))?;
    }
    Ok(())
}

fn create_ensure_global_cache_is_disabled_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to disable Yarn global cache")
        .body(formatdoc! {"
            The Heroku Node.js buildpack was unable to disable the Yarn global cache.
        "})
        .debug_info(error.to_string())
        .create()
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
        .error_type(ErrorType::Internal)
        .header("Failed to read configured Yarn cache directory")
        .body(formatdoc! {"
            The Heroku Node.js buildpack was unable to read the configuration for the Yarn cache directory.
        "})
        .debug_info(error.to_string())
        .create()
}

fn set_cache_folder_config(
    env: &Env,
    version: &Version,
    cache_dir: &Path,
) -> Result<(), ErrorMessage> {
    print::sub_stream_cmd(
        Command::new("yarn")
            .envs(env)
            .arg("config")
            .arg("set")
            .arg(match version.major() {
                1 => "cache-folder",
                _ => "cacheFolder",
            })
            .arg(cache_dir.to_string_lossy().to_string()),
    )
    .map(|_| ())
    .map_err(|e| create_set_cache_folder_config_error(&e))
}

fn create_set_cache_folder_config_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to configure Yarn cache directory")
        .body(formatdoc! {"
            An unexpected error occurred while configuring the Yarn cache directory.
        " })
        .debug_info(error.to_string())
        .create()
}

// A yarn cache is populated if it exists and has non-hidden files, indicating a zero-install mode
// should be employed.
fn is_yarn_zero_install_mode(yarn_cache: &Path) -> bool {
    yarn_cache
        .read_dir()
        .map(|mut contents| {
            contents.any(|entry| {
                entry
                    .map(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
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
    node_linker: Option<&NodeLinker>,
) -> BuildpackResult<PathBuf> {
    let new_metadata = YarnCacheDirectoryLayerMetadata {
        layer_version: YARN_CACHE_DIRECTORY_LAYER_VERSION.to_string(),
        cache_usage_count: 0.0,
        yarn_major_version: version.major().to_string(),
    };

    let yarn_cache_layer = context.cached_layer(
        layer_name!("yarn_cache"),
        CachedLayerDefinition {
            build: true,
            // In Plug'n'Play mode, Yarn resolves packages directly from the cache so it must be present at launch
            launch: matches!(node_linker, Some(NodeLinker::Pnp)),
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &YarnCacheDirectoryLayerMetadata, _| {
                let is_reusable = old_metadata.yarn_major_version
                    == new_metadata.yarn_major_version
                    && old_metadata.layer_version == new_metadata.layer_version
                    && old_metadata.cache_usage_count < YARN_CACHE_DIRECTORY_MAX_CACHE_USAGE_COUNT;
                if is_reusable {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    match yarn_cache_layer.state {
        LayerState::Restored { .. } => {
            print::sub_bullet("Restoring yarn dependency cache");
        }
        LayerState::Empty { cause } => {
            if let EmptyLayerCause::RestoredLayerAction { .. } = cause {
                print::sub_bullet("Clearing yarn dependency cache");
            }
            yarn_cache_layer.write_metadata(YarnCacheDirectoryLayerMetadata {
                cache_usage_count: new_metadata.cache_usage_count + 1.0,
                ..new_metadata
            })?;
        }
    }

    Ok(yarn_cache_layer.path())
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
    fn write_vendored_yarn_link_error() {
        assert_error_snapshot(&create_write_vendored_yarn_link_error(
            &PathBuf::from("/path/to/yarn"),
            &std::io::Error::other("Not found"),
        ));
    }

    #[test]
    fn ensure_global_cache_is_disabled_error() {
        assert_error_snapshot(&create_ensure_global_cache_is_disabled_error(
            &create_cmd_error("yarn config set enableGlobalCache false"),
        ));
    }

    #[test]
    fn get_cache_folder_config_error() {
        assert_error_snapshot(&create_get_cache_folder_config_error(&create_cmd_error(
            "yarn config get cacheFolder",
        )));
    }

    #[test]
    fn set_cache_folder_config_error() {
        assert_error_snapshot(&create_set_cache_folder_config_error(&create_cmd_error(
            "yarn config set cacheFolder /path/to/cache",
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
}
