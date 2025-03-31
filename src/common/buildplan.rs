use libcnb::data::buildpack_plan::BuildpackPlan;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct NodeBuildScriptsMetadata {
    pub enabled: Option<bool>,
}

pub(crate) const NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME: &str = "node_build_scripts";
const NODE_BUILD_SCRIPTS_METADATA_ENABLED_KEY: &str = "enabled";

pub(crate) fn read_node_build_scripts_metadata(
    buildpack_plan: &BuildpackPlan,
) -> Result<NodeBuildScriptsMetadata, NodeBuildScriptsMetadataError> {
    buildpack_plan
        .entries
        .iter()
        .filter(|entry| entry.name == NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
        .try_fold(
            NodeBuildScriptsMetadata::default(),
            |mut node_build_hooks_metadata, entry| {
                match entry.metadata.get(NODE_BUILD_SCRIPTS_METADATA_ENABLED_KEY) {
                    Some(toml::Value::Boolean(enabled)) => {
                        node_build_hooks_metadata.enabled = Some(*enabled);
                    }
                    Some(value) => {
                        Err(NodeBuildScriptsMetadataError::InvalidEnabledValue(
                            value.clone(),
                        ))?;
                    }
                    None => {}
                }
                Ok(node_build_hooks_metadata)
            },
        )
}

#[derive(Debug)]
pub(crate) enum NodeBuildScriptsMetadataError {
    InvalidEnabledValue(toml::Value),
}

#[cfg(test)]
mod test {
    use super::*;
    use libcnb::data::buildpack_plan::Entry;
    use toml::{toml, Table};

    #[test]
    fn read_node_build_scripts_when_buildpack_plan_contains_no_entries() {
        let buildpack_plan = BuildpackPlan { entries: vec![] };
        assert_eq!(
            read_node_build_scripts_metadata(&buildpack_plan).unwrap(),
            NodeBuildScriptsMetadata::default()
        );
    }

    #[test]
    fn read_node_build_scripts_when_entry_is_present_with_no_metadata() {
        let buildpack_plan = BuildpackPlan {
            entries: vec![Entry {
                name: NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME.to_string(),
                metadata: Table::new(),
            }],
        };
        assert_eq!(
            read_node_build_scripts_metadata(&buildpack_plan).unwrap(),
            NodeBuildScriptsMetadata::default()
        );
    }

    #[test]
    fn read_node_build_scripts_when_entry_is_present_and_metadata_is_declared() {
        let buildpack_plan = BuildpackPlan {
            entries: vec![Entry {
                name: NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME.to_string(),
                metadata: toml! {
                    enabled = false
                },
            }],
        };
        assert_eq!(
            read_node_build_scripts_metadata(&buildpack_plan).unwrap(),
            NodeBuildScriptsMetadata {
                enabled: Some(false)
            }
        );
    }

    #[test]
    fn read_node_build_scripts_when_multiple_entries_are_present_and_metadata_is_declared() {
        let buildpack_plan = BuildpackPlan {
            entries: vec![
                Entry {
                    name: NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME.to_string(),
                    metadata: toml! {
                        enabled = false
                    },
                },
                Entry {
                    name: NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME.to_string(),
                    metadata: toml! {
                        enabled = true
                    },
                },
                Entry {
                    name: NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME.to_string(),
                    metadata: Table::new(),
                },
            ],
        };
        assert_eq!(
            read_node_build_scripts_metadata(&buildpack_plan).unwrap(),
            NodeBuildScriptsMetadata {
                enabled: Some(true)
            }
        );
    }

    #[test]
    fn read_node_build_scripts_when_entry_contains_invalid_metadata() {
        let buildpack_plan = BuildpackPlan {
            entries: vec![Entry {
                name: NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME.to_string(),
                metadata: toml! {
                    enabled = 0
                },
            }],
        };
        match read_node_build_scripts_metadata(&buildpack_plan).unwrap_err() {
            NodeBuildScriptsMetadataError::InvalidEnabledValue(_) => {}
        }
    }
}
