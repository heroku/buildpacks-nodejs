use crate::BuildpackBuildContext;
use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use bullet_stream::style;
use indoc::formatdoc;
use libcnb::data::buildpack::BuildpackId;
use libcnb::data::buildpack_plan::BuildpackPlan;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use toml::Table;
use toml_edit::{DocumentMut, TableLike};

pub(crate) const NAMESPACED_CONFIG: &str = "com.heroku.buildpacks.nodejs";

#[derive(Debug, PartialEq)]
pub(crate) struct ConfigValue<T> {
    pub(crate) value: T,
    pub(crate) source: ConfigValueSource,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ConfigValueSource {
    Buildplan(Option<BuildpackId>),
    ProjectToml,
}

impl Display for ConfigValueSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigValueSource::Buildplan(Some(buildpack_id)) => {
                write!(f, "buildplan from {buildpack_id}")
            }
            ConfigValueSource::Buildplan(None) => {
                write!(f, "buildplan from unidentified buildpack")
            }
            ConfigValueSource::ProjectToml => write!(f, "project.toml"),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct BuildpackConfig {
    pub(crate) build_scripts_enabled: Option<ConfigValue<bool>>,
    pub(crate) prune_dev_dependencies: Option<ConfigValue<bool>>,
    errors: Vec<String>,
}

/// Buildpack configuration can come from several sources such as:
/// - buildplan entries provided by later buildpacks
/// - user configuration defined in project.toml
///
/// To avoid conflict with other configuration in project.toml, a namespace must be used. E.g.;
///
/// ```toml
/// [com.heroku.buildpacks.nodejs]
/// enabled = true
/// actions.prune_dev_dependencies = false
/// ```
///
/// This namespacing is not necessary for buildplan entries as the contributing buildpack already has
/// to assign the buildplan to a name. To avoid conflicts with other buildpack names, we use the
/// buildpack id as the buildplan name (i.e.; `heroku/nodejs`) and the configuration is read from the
/// `[metadata]` section of the buildplan entry.
///
/// When multiple config sources are provided, they are merged in order of precedence:
/// - Buildplans (first to last)
/// - project.toml
impl BuildpackConfig {
    pub(crate) fn merge<I>(configs: I) -> BuildpackConfig
    where
        I: IntoIterator<Item = BuildpackConfig>,
    {
        let mut merged_config = BuildpackConfig::default();
        for config in configs {
            let BuildpackConfig {
                build_scripts_enabled,
                prune_dev_dependencies,
                errors,
            } = config;
            if build_scripts_enabled.is_some() {
                merged_config.build_scripts_enabled = build_scripts_enabled;
            }
            if prune_dev_dependencies.is_some() {
                merged_config.prune_dev_dependencies = prune_dev_dependencies;
            }
            merged_config.errors.extend(errors);
        }
        merged_config
    }
}

impl TryFrom<&BuildpackBuildContext> for BuildpackConfig {
    type Error = ErrorMessage;

    fn try_from(value: &BuildpackBuildContext) -> Result<Self, Self::Error> {
        let buildpack_id = &value.buildpack_descriptor.buildpack.id;
        let buildpack_plan = &value.buildpack_plan;
        let project_toml = &value.app_dir.join("project.toml");
        BuildpackConfig::try_from((buildpack_id, buildpack_plan, project_toml))
    }
}

impl TryFrom<(&BuildpackId, &BuildpackPlan, &PathBuf)> for BuildpackConfig {
    type Error = ErrorMessage;

    fn try_from(value: (&BuildpackId, &BuildpackPlan, &PathBuf)) -> Result<Self, Self::Error> {
        let (buildpack_id, buildpack_plan, project_toml) = value;
        let buildpack_plan_config = BuildpackConfig::try_from((buildpack_id, buildpack_plan))?;
        let project_toml_config = BuildpackConfig::try_from(project_toml)?;
        Ok(BuildpackConfig::merge([
            buildpack_plan_config,
            project_toml_config,
        ]))
    }
}

impl TryFrom<(&BuildpackId, &BuildpackPlan)> for BuildpackConfig {
    type Error = ErrorMessage;

    fn try_from(value: (&BuildpackId, &BuildpackPlan)) -> Result<Self, Self::Error> {
        let (buildpack_id, buildpack_plan) = value;
        let buildpack_plan_configs = buildpack_plan
            .entries
            .iter()
            .filter_map(|entry| {
                if entry.name == buildpack_id.to_string() {
                    let source_id = entry
                        .metadata
                        .get("_")
                        .and_then(|v| v.as_table())
                        .and_then(|table| table.get("source"))
                        .and_then(|source| source.as_str())
                        .and_then(|source| BuildpackId::from_str(source).ok());
                    Some(BuildpackConfig::try_from((
                        &ConfigValueSource::Buildplan(source_id),
                        &entry.metadata,
                    )))
                } else {
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(BuildpackConfig::merge(buildpack_plan_configs))
    }
}

impl TryFrom<&PathBuf> for BuildpackConfig {
    type Error = ErrorMessage;

    fn try_from(value: &PathBuf) -> Result<Self, Self::Error> {
        match std::fs::read_to_string(value) {
            Ok(contents) => {
                let doc = DocumentMut::from_str(&contents)
                    .map_err(|e| create_parse_project_toml_error_message(&e))?;
                match doc.as_item().as_table_like() {
                    Some(mut current_table) => {
                        for name in NAMESPACED_CONFIG.split('.') {
                            current_table =
                                match current_table.get(name).and_then(|v| v.as_table_like()) {
                                    Some(table_like) => table_like,
                                    None => return Ok(BuildpackConfig::default()),
                                };
                        }
                        BuildpackConfig::try_from((&ConfigValueSource::ProjectToml, current_table))
                    }
                    None => Ok(BuildpackConfig::default()),
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(BuildpackConfig::default())
            }
            Err(error) => Err(create_read_project_toml_error_message(&error)),
        }
    }
}

impl TryFrom<(&ConfigValueSource, &Table)> for BuildpackConfig {
    type Error = ErrorMessage;

    fn try_from(value: (&ConfigValueSource, &Table)) -> Result<Self, Self::Error> {
        let (source, table) = value;
        let toml_contents =
            toml::to_string(table).expect("Buildplan TOML data should be serializable");
        let doc =
            DocumentMut::from_str(&toml_contents).expect("Buildplan contents should be valid TOML");
        let table_like = doc
            .as_item()
            .as_table_like()
            .expect("toml doc should be table-like");
        BuildpackConfig::try_from((source, table_like))
    }
}

impl TryFrom<(&ConfigValueSource, &dyn TableLike)> for BuildpackConfig {
    type Error = ErrorMessage;

    fn try_from(value: (&ConfigValueSource, &dyn TableLike)) -> Result<Self, Self::Error> {
        let (source, table) = value;
        let build_scripts_enabled =
            table
                .get("enabled")
                .and_then(toml_edit::Item::as_bool)
                .map(|value| ConfigValue {
                    value,
                    source: source.clone(),
                });
        // TODO: these config sources should be aligned
        let prune_dev_dependencies = match source {
            ConfigValueSource::Buildplan(_) => table
                .get("skip_pruning")
                .and_then(toml_edit::Item::as_bool)
                .map(|value| ConfigValue {
                    value: !value,
                    source: source.clone(),
                }),
            ConfigValueSource::ProjectToml => table
                .get("actions")
                .and_then(|v| v.as_table_like())
                .and_then(|v| v.get("prune_dev_dependencies"))
                .and_then(toml_edit::Item::as_bool)
                .map(|value| ConfigValue {
                    value,
                    source: source.clone(),
                }),
        };
        Ok(BuildpackConfig {
            build_scripts_enabled,
            prune_dev_dependencies,
            errors: Vec::new(),
        })
    }
}

fn create_read_project_toml_error_message(error: &std::io::Error) -> ErrorMessage {
    let project_toml = style::value("project.toml");
    error_message()
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error reading {project_toml}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {project_toml} to read configuration data but \
            the file can't be read.

            Suggestions:
            - Ensure the file has read permissions.
        " })
        .debug_info(error.to_string())
        .create()
}

fn create_parse_project_toml_error_message(error: &toml_edit::TomlError) -> ErrorMessage {
    let project_toml = style::value("project.toml");
    let toml_spec_url = style::url("https://toml.io/en/v1.0.0");
    error_message()
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error parsing {project_toml}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {project_toml} to read configuration but \
            the file isn't valid TOML.

            Suggestions:
            - Ensure the file follows the TOML format described at {toml_spec_url}
        " })
        .debug_info(error.to_string())
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::assert_error_snapshot;
    use libcnb::data::buildpack_id;
    use libcnb::data::buildpack_plan::Entry;
    use std::fmt::Write;
    use toml::Value;

    #[test]
    fn config_from_nothing() {
        let config = multisource_buildpack_config().build().unwrap();
        assert_eq!(config.build_scripts_enabled, None);
        assert_eq!(config.prune_dev_dependencies, None);
    }

    #[test]
    fn config_from_project_toml() {
        let config = multisource_buildpack_config()
            .project_toml(|config| {
                config
                    .build_scripts_enabled(true)
                    .prune_dev_dependencies(false)
            })
            .build()
            .unwrap();
        assert_eq!(
            config.build_scripts_enabled,
            Some(ConfigValue {
                value: true,
                source: ConfigValueSource::ProjectToml
            })
        );
        assert_eq!(
            config.prune_dev_dependencies,
            Some(ConfigValue {
                value: false,
                source: ConfigValueSource::ProjectToml
            })
        );
    }

    #[test]
    fn config_from_buildplan_with_no_source_id() {
        let config = multisource_buildpack_config()
            .buildpack_plan(|config| config.build_scripts_enabled(true).skip_pruning(true))
            .build()
            .unwrap();
        assert_eq!(
            config.build_scripts_enabled,
            Some(ConfigValue {
                value: true,
                source: ConfigValueSource::Buildplan(None)
            })
        );
        assert_eq!(
            config.prune_dev_dependencies,
            Some(ConfigValue {
                value: false,
                source: ConfigValueSource::Buildplan(None)
            })
        );
    }

    #[test]
    fn config_from_multiple_buildplans() {
        let config = multisource_buildpack_config()
            .buildpack_plan(|config| {
                config
                    .source_id("test/plan-1")
                    .build_scripts_enabled(true)
                    .skip_pruning(true)
            })
            .buildpack_plan(|config| {
                config
                    .source_id("test/plan-2")
                    .build_scripts_enabled(false)
                    .skip_pruning(false)
            })
            .buildpack_plan(|config| config) // this config is empty and should be ignored
            .build()
            .unwrap();
        assert_eq!(
            config.build_scripts_enabled,
            Some(ConfigValue {
                value: false,
                source: ConfigValueSource::Buildplan(Some(buildpack_id!("test/plan-2")))
            })
        );
        assert_eq!(
            config.prune_dev_dependencies,
            Some(ConfigValue {
                value: true,
                source: ConfigValueSource::Buildplan(Some(buildpack_id!("test/plan-2")))
            })
        );
    }

    #[test]
    fn config_from_buildplan_and_project_toml() {
        let config = multisource_buildpack_config()
            .buildpack_plan(|config| config.source_id("test/plan-1").skip_pruning(true))
            .buildpack_plan(|config| config.source_id("test/plan-2")) // this config is empty and should be ignored
            .project_toml(|config| config.build_scripts_enabled(false))
            .build()
            .unwrap();
        assert_eq!(
            config.build_scripts_enabled,
            Some(ConfigValue {
                value: false,
                source: ConfigValueSource::ProjectToml
            })
        );
        assert_eq!(
            config.prune_dev_dependencies,
            Some(ConfigValue {
                value: false,
                source: ConfigValueSource::Buildplan(Some(buildpack_id!("test/plan-1")))
            })
        );
    }

    #[test]
    fn config_when_project_toml_is_invalid_toml() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_toml_path = temp_dir.path().join("project.toml");
        std::fs::write(&project_toml_path, "[invalid toml").unwrap();
        let error = BuildpackConfig::try_from(&project_toml_path).unwrap_err();
        assert_error_snapshot(&error);
    }

    #[test]
    fn config_when_project_toml_cannot_be_read() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_toml_path = temp_dir.path().join("project.toml");
        std::fs::create_dir(&project_toml_path).unwrap();
        let error = BuildpackConfig::try_from(&project_toml_path).unwrap_err();
        assert_error_snapshot(&error);
    }

    #[test]
    fn config_when_project_toml_exists_but_does_not_contain_buildpack_config() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_toml_path = temp_dir.path().join("project.toml");
        std::fs::write(&project_toml_path, "key = \"value\"").unwrap();

        let buildpack_id = buildpack_id!("heroku/nodejs");
        let buildpack_plan = BuildpackPlan {
            entries: Vec::from([Entry {
                name: buildpack_id.to_string(),
                metadata: BuildplanConfig::builder()
                    .skip_pruning(false)
                    .build_scripts_enabled(false)
                    .build()
                    .into_toml(),
            }]),
        };

        let config =
            BuildpackConfig::try_from((&buildpack_id, &buildpack_plan, &project_toml_path))
                .unwrap();
        assert_eq!(
            config.build_scripts_enabled,
            Some(ConfigValue {
                value: false,
                source: ConfigValueSource::Buildplan(None)
            })
        );
        assert_eq!(
            config.prune_dev_dependencies,
            Some(ConfigValue {
                value: true,
                source: ConfigValueSource::Buildplan(None)
            })
        );
    }

    #[bon::builder(finish_fn = build)]
    fn multisource_buildpack_config(
        #[builder(field)] buildpack_plan_configs: Vec<BuildplanConfig>,
        #[builder(field)] project_toml: Option<ProjectTomlConfig>,
    ) -> Result<BuildpackConfig, ErrorMessage> {
        let buildpack_id = buildpack_id!("heroku/nodejs");
        let buildpack_plan = BuildpackPlan {
            entries: buildpack_plan_configs
                .into_iter()
                .map(|config| Entry {
                    name: buildpack_id.to_string(),
                    metadata: config.into_toml(),
                })
                .collect(),
        };

        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_toml_path = temp_dir.path().join("project.toml");
        if let Some(config) = project_toml {
            let toml = toml::to_string(&config.into_toml()).unwrap();
            std::fs::write(&project_toml_path, toml).unwrap();
        }

        BuildpackConfig::try_from((&buildpack_id, &buildpack_plan, &project_toml_path))
    }

    impl<S: multisource_buildpack_config_builder::State> MultisourceBuildpackConfigBuilder<S> {
        fn buildpack_plan<C>(
            mut self,
            value: impl FnOnce(BuildplanConfigBuilder) -> BuildplanConfigBuilder<C>,
        ) -> Self
        where
            C: buildplan_config_builder::IsComplete,
        {
            self.buildpack_plan_configs
                .push(value(BuildplanConfig::builder()).build());
            self
        }

        fn project_toml<C>(
            mut self,
            value: impl FnOnce(ProjectTomlConfigBuilder) -> ProjectTomlConfigBuilder<C>,
        ) -> Self
        where
            C: project_toml_config_builder::IsComplete,
        {
            self.project_toml = Some(value(ProjectTomlConfig::builder()).build());
            self
        }
    }

    #[derive(bon::Builder)]
    struct ProjectTomlConfig {
        build_scripts_enabled: Option<bool>,
        prune_dev_dependencies: Option<bool>,
    }

    impl ProjectTomlConfig {
        fn into_toml(self) -> Table {
            let mut toml = String::new();
            let _ = writeln!(toml, "[{NAMESPACED_CONFIG}]");
            if let Some(build_scripts_enabled) = self.build_scripts_enabled {
                let _ = writeln!(toml, "enabled = {build_scripts_enabled}");
            }
            if let Some(prune_dev_dependencies) = self.prune_dev_dependencies {
                let _ = writeln!(
                    toml,
                    "actions.prune_dev_dependencies = {prune_dev_dependencies}"
                );
            }
            toml::from_str(&toml).unwrap()
        }
    }

    #[derive(bon::Builder)]
    #[builder(on(String, into))]
    struct BuildplanConfig {
        build_scripts_enabled: Option<bool>,
        skip_pruning: Option<bool>,
        source_id: Option<String>,
    }

    impl BuildplanConfig {
        fn into_toml(self) -> Table {
            let mut table = Table::new();
            if let Some(build_scripts_enabled) = self.build_scripts_enabled {
                table.insert("enabled".into(), Value::Boolean(build_scripts_enabled));
            }
            if let Some(skip_pruning) = self.skip_pruning {
                table.insert("skip_pruning".into(), Value::Boolean(skip_pruning));
            }
            if let Some(source_id) = self.source_id {
                let mut source_table = Table::new();
                source_table.insert("source".into(), Value::String(source_id));
                table.insert("_".into(), Value::Table(source_table));
            }
            table
        }
    }
}
