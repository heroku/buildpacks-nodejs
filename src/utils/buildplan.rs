use libcnb::data::buildpack_plan::BuildpackPlan;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct NodeBuildScriptsMetadata {
    pub(crate) enabled: Option<bool>,
    pub(crate) skip_pruning: Option<bool>,
}

pub(crate) const NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME: &str = "heroku/nodejs";
const NODE_BUILD_SCRIPTS_METADATA_ENABLED_KEY: &str = "enabled";
const NODE_BUILD_SCRIPTS_METADATA_SKIP_PRUNING_KEY: &str = "skip_pruning";

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
                match entry
                    .metadata
                    .get(NODE_BUILD_SCRIPTS_METADATA_SKIP_PRUNING_KEY)
                {
                    Some(toml::Value::Boolean(enabled)) => {
                        node_build_hooks_metadata.skip_pruning = Some(*enabled);
                    }
                    Some(value) => {
                        Err(NodeBuildScriptsMetadataError::InvalidSkipPruningValue(
                            value.clone(),
                        ))?;
                    }
                    None => {
                        // As the front-end buildpacks are the only ones that are likely to
                        // using the buildplan metadata, if the build scripts are disabled,
                        // there's a good chance that we need to also skip pruning.
                        if node_build_hooks_metadata.enabled == Some(false) {
                            node_build_hooks_metadata.skip_pruning = Some(true);
                        } else {
                            node_build_hooks_metadata.skip_pruning = None;
                        }
                    }
                }
                Ok(node_build_hooks_metadata)
            },
        )
}

#[derive(Debug)]
pub(crate) enum NodeBuildScriptsMetadataError {
    InvalidEnabledValue(toml::Value),
    InvalidSkipPruningValue(toml::Value),
}

#[cfg(test)]
mod test {
    use super::*;
    use libcnb::data::buildpack_plan::{BuildpackPlan, Entry};
    use toml::{Table, toml};

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
                    skip_pruning = false
                },
            }],
        };
        assert_eq!(
            read_node_build_scripts_metadata(&buildpack_plan).unwrap(),
            NodeBuildScriptsMetadata {
                enabled: Some(false),
                skip_pruning: Some(false)
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
                enabled: Some(true),
                skip_pruning: None
            }
        );
    }

    #[test]
    fn read_node_build_scripts_when_entry_contains_invalid_metadata_for_enabled() {
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
            e @ NodeBuildScriptsMetadataError::InvalidSkipPruningValue(_) => {
                panic!("Unexpected error: {e:?}")
            }
        }
    }

    #[test]
    fn read_node_build_scripts_when_entry_contains_invalid_metadata_for_skip_pruning() {
        let buildpack_plan = BuildpackPlan {
            entries: vec![Entry {
                name: NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME.to_string(),
                metadata: toml! {
                    enabled = false
                    skip_pruning = 0
                },
            }],
        };
        match read_node_build_scripts_metadata(&buildpack_plan).unwrap_err() {
            NodeBuildScriptsMetadataError::InvalidSkipPruningValue(_) => {}
            e @ NodeBuildScriptsMetadataError::InvalidEnabledValue(_) => {
                panic!("Unexpected error: {e:?}")
            }
        }
    }
}
