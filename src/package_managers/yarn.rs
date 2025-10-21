use crate::utils::error_handling::ErrorType::Internal;
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
use libcnb::layer::UncachedLayerDefinition;
use libcnb::layer_env::Scope;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    std::os::unix::fs::symlink(full_yarn_path, yarn)
        .map_err(|e| create_write_vendored_yarn_link_error(yarn_path, &e))?;

    let layer_env = &bin_layer.read_env()?;

    env.clone_from(&layer_env.apply(Scope::Build, env));

    Ok(())
}

fn create_write_vendored_yarn_link_error(yarn_path: &Path, error: &std::io::Error) -> ErrorMessage {
    let yarn_path = style::value(yarn_path.to_string_lossy());
    let yarn = style::value("yarn");
    error_message()
        .error_type(Internal)
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
            .error_type(Internal)
            .header("Failed to determine Yarn version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current Yarn version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        VersionCommandError::Parse(stdout, e) => error_message()
            .error_type(Internal)
            .header("Failed to parse Yarn version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing Yarn version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

pub(crate) fn install_dependencies(
    _context: &BuildpackBuildContext,
    env: &Env,
    version: &Version,
) -> BuildpackResult<()> {
    print::bullet("Setting up yarn dependency cache");

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
        .map_err(|e| create_yarn_disable_global_cache_error(&e))?;
    }

    // TODO: implement yarn install

    Ok(())
}

fn create_yarn_disable_global_cache_error(error: &fun_run::CmdError) -> ErrorMessage {
    error_message()
        .error_type(ErrorType::Internal)
        .header("Failed to disable Yarn global cache")
        .body(formatdoc! {"
            The Heroku Node.js buildpack was unable to disable the Yarn global cache.
        "})
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
            crate::BuildpackError::BuildpackError(crate::NodeJsBuildpackError::Message(
                message,
            )) => {
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
            crate::BuildpackError::BuildpackError(crate::NodeJsBuildpackError::Message(
                message,
            )) => {
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
            crate::BuildpackError::BuildpackError(crate::NodeJsBuildpackError::Message(
                message,
            )) => {
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
    fn yarn_disable_global_cache_error() {
        assert_error_snapshot(&create_yarn_disable_global_cache_error(&create_cmd_error(
            "yarn config set enableGlobalCache false",
        )));
    }
}
