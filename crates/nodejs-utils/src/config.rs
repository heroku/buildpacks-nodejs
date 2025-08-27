use crate::error_handling::ErrorType::UserFacing;
use crate::error_handling::{
    error_message, ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue,
};
use bullet_stream::style;
use indoc::formatdoc;
use std::path::Path;
use std::str::FromStr;
use toml_edit::TableLike;

const CONFIG_NAMESPACE: [&str; 5] = ["com", "heroku", "buildpacks", "nodejs", "actions"];
const PRUNE_DEV_DEPENDENCY_CONFIG: &str = "prune_dev_dependencies";

pub fn read_prune_dev_dependencies_from_project_toml(
    project_toml_path: &Path,
) -> Result<Option<bool>, ConfigError> {
    let project_toml_exists = project_toml_path
        .try_exists()
        .map_err(ConfigError::CheckExists)?;

    if !project_toml_exists {
        return Ok(None);
    }

    let project_toml = std::fs::read_to_string(project_toml_path)
        .map_err(ConfigError::ReadProjectToml)
        .and_then(|contents| {
            toml_edit::DocumentMut::from_str(&contents).map_err(ConfigError::ParseProjectToml)
        })?;

    match get_buildpack_namespaced_config(&project_toml)? {
        Some(namespaced_config) => match namespaced_config.get(PRUNE_DEV_DEPENDENCY_CONFIG) {
            Some(config) => config.as_bool().map(Some).ok_or(ConfigError::WrongType),
            None => Ok(None),
        },
        None => Ok(None),
    }
}

fn get_buildpack_namespaced_config(
    doc: &toml_edit::DocumentMut,
) -> Result<Option<&dyn TableLike>, ConfigError> {
    let mut current_table = doc
        .as_item()
        .as_table_like()
        .ok_or(ConfigError::ExpectedTomlTable)?;
    for name in CONFIG_NAMESPACE {
        current_table = match current_table.get(name) {
            Some(item) => match item.as_table_like() {
                Some(table) => table,
                None => return Err(ConfigError::ExpectedTomlTable),
            },
            None => return Ok(None),
        };
    }
    Ok(Some(current_table))
}

#[derive(Debug)]
pub enum ConfigError {
    CheckExists(std::io::Error),
    ReadProjectToml(std::io::Error),
    ParseProjectToml(toml_edit::TomlError),
    ExpectedTomlTable,
    WrongType,
}

impl From<ConfigError> for ErrorMessage {
    fn from(value: ConfigError) -> Self {
        let project_toml = style::value("project.toml");
        let toml_spec_url = style::url("https://toml.io/en/v1.0.0");
        let config_table_key = style::value(format!("[{}]", CONFIG_NAMESPACE.to_vec().join(".")));
        let config_key = style::value(PRUNE_DEV_DEPENDENCY_CONFIG);

        match value {
            ConfigError::CheckExists(e) => error_message()
                .error_type(ErrorType::Internal)
                .header("Failed project.toml existence check")
                .body(formatdoc! { "
                    An unexpected I/O error occurred while checking if {project_toml} exists in the \
                    root of the application.
                "})
                .debug_info(e.to_string())
                .create(),

            ConfigError::ReadProjectToml(e) => error_message()
                .error_type(ErrorType::Internal)
                .header("Failed to read project.toml")
                .body(formatdoc! { "
                    This buildpack will read from {project_toml} if the file is present in the root of \
                    the application but an unexpected I/O error occurred during this operation.
                "})
                .debug_info(e.to_string())
                .create(),

            ConfigError::ParseProjectToml(e) => error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header("Failed to parse project.toml")
                .body(formatdoc! { "
                    This buildpack reads configuration from {project_toml} to complete the build, but \
                    this file isn't a valid TOML file.

                    Suggestions:
                    - Ensure the file follows the TOML format described at {toml_spec_url}
                "})
                .debug_info(e.to_string())
                .create(),

            ConfigError::ExpectedTomlTable => error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header("Error parsing project.toml with invalid key")
                .body(formatdoc! { "
                    This buildpack reads the configuration from {project_toml} to complete \
                    the build, but the configuration for the key {config_table_key} isn't the \
                    correct type. The value of this key must be a TOML table.

                    Suggestions:
                    - See the TOML documentation for more details on the TOML table type at \
                    {toml_spec_url}
                " })
                .maybe_debug_info(None::<String>)
                .create(),

            ConfigError::WrongType => error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header("Error parsing project.toml with invalid configuration type")
                .body(formatdoc! { "
                    This buildpack reads the configuration from {project_toml} to complete \
                    the build, but the configuration value for {config_key} in the section \
                    {config_table_key} isn't the correct type. The value of this key must be a boolean.

                    Suggestions:
                    - See the TOML documentation for more details on the TOML boolean type at \
                    {toml_spec_url}
                " })
                .maybe_debug_info(None::<String>)
                .create(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use indoc::indoc;
    use insta::{assert_snapshot, with_settings};
    use test_support::test_name;
    use toml::toml;

    #[test]
    fn read_prune_dev_dependencies_config_from_project_toml_when_true() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        let config = toml::to_string(&toml! {
            [com.heroku.buildpacks.nodejs.actions]
            prune_dev_dependencies = true
        })
        .unwrap();
        std::fs::write(&project_toml, &config).unwrap();

        assert_eq!(
            read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap(),
            Some(true)
        );
    }

    #[test]
    fn read_prune_dev_dependencies_config_from_project_toml_when_false() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        let config = toml::to_string(&toml! {
            [com.heroku.buildpacks.nodejs.actions]
            prune_dev_dependencies = false
        })
        .unwrap();
        std::fs::write(&project_toml, &config).unwrap();

        assert_eq!(
            read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap(),
            Some(false)
        );
    }

    #[test]
    fn read_prune_dev_dependencies_config_from_project_toml_when_no_value_set() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        let config = toml::to_string(&toml! {
            [com.heroku.buildpacks.nodejs.actions]
            some_other_config = "test"

            [some.other.namespace]
            prune_dev_dependencies = true
        })
        .unwrap();
        std::fs::write(&project_toml, &config).unwrap();

        assert_eq!(
            read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap(),
            None
        );
    }

    #[test]
    fn read_prune_dev_dependencies_config_when_no_project_toml_is_present() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        assert_eq!(
            read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap(),
            None
        );
    }

    #[test]
    fn read_prune_dev_dependencies_config_when_there_is_an_error_reading_project_toml() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        std::fs::create_dir(&project_toml).unwrap();
        match read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap_err() {
            ConfigError::ReadProjectToml(_) => {}
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn read_prune_dev_dependencies_config_when_there_is_an_error_parsing_project_toml() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        let config = indoc! { "
            [some.other.namespace]
            some_other_config =
        " };
        std::fs::write(&project_toml, config).unwrap();
        match read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap_err() {
            ConfigError::ParseProjectToml(_) => {}
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn read_prune_dev_dependencies_config_when_there_is_an_error_getting_the_namespaced_config() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        let config = toml::to_string(&toml! {
            [com.heroku.buildpacks.nodejs]
            actions = "i should be a table"
        })
        .unwrap();
        std::fs::write(&project_toml, config).unwrap();
        match read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap_err() {
            ConfigError::ExpectedTomlTable => {}
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn read_prune_dev_dependencies_config_when_there_is_an_error_with_the_config_type() {
        let app_dir = tempfile::tempdir().unwrap();
        let project_toml = app_dir.path().join("project.toml");
        let config = toml::to_string(&toml! {
            [com.heroku.buildpacks.nodejs.actions]
            prune_dev_dependencies = "i should be a boolean value"
        })
        .unwrap();
        std::fs::write(&project_toml, config).unwrap();
        match read_prune_dev_dependencies_from_project_toml(&project_toml).unwrap_err() {
            ConfigError::WrongType => {}
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn test_config_error_check_exists() {
        assert_config_error_snapshot(ConfigError::CheckExists(std::io::Error::other(
            "Test I/O Error",
        )));
    }

    #[test]
    fn test_config_error_read_project_toml() {
        assert_config_error_snapshot(ConfigError::ReadProjectToml(std::io::Error::other(
            "Test I/O Error",
        )));
    }

    #[test]
    fn test_config_error_parse_project_toml() {
        assert_config_error_snapshot(ConfigError::ParseProjectToml(create_toml_edit_error()));
    }

    #[test]
    fn test_config_error_expected_toml_table() {
        assert_config_error_snapshot(ConfigError::ExpectedTomlTable);
    }

    #[test]
    fn test_config_error_wrong_type() {
        assert_config_error_snapshot(ConfigError::WrongType);
    }

    fn assert_config_error_snapshot(error: impl Into<ErrorMessage>) {
        let error_message = strip_ansi(error.into().to_string());
        let test_name = format!(
            "errors__{}",
            test_name()
                .split("::")
                .last()
                .unwrap()
                .trim_start_matches("test")
        );
        with_settings!({
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            assert_snapshot!(test_name, error_message);
        });
    }

    fn create_toml_edit_error() -> toml_edit::TomlError {
        toml_edit::DocumentMut::from_str("[[artifacts").unwrap_err()
    }
}
