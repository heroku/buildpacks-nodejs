use crate::{NodeJsEngineBuildpack, NodeJsEngineBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::UncachedLayerDefinition;
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use thiserror::Error;

pub(crate) fn attach_runtime_metrics(
    context: &BuildContext<NodeJsEngineBuildpack>,
) -> Result<(), libcnb::Error<NodeJsEngineBuildpackError>> {
    let web_env_layer = context.uncached_layer(
        layer_name!("node_runtime_metrics"),
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;

    let metrics_script = web_env_layer.path().join("metrics_collector.cjs");

    std::fs::write(
        &metrics_script,
        include_bytes!("../node_runtime_metrics/metrics_collector.cjs"),
    )
    .map_err(NodeRuntimeMetricsError::WriteMetricsScript)?;

    web_env_layer.write_env(
        LayerEnv::new()
            .chainable_insert(
                Scope::Launch,
                ModificationBehavior::Delimiter,
                "NODE_OPTIONS",
                " ",
            )
            .chainable_insert(
                Scope::Launch,
                ModificationBehavior::Append,
                "NODE_OPTIONS",
                format!("--require {}", metrics_script.display()),
            ),
    )?;

    Ok(())
}

#[derive(Debug, Error)]
pub(crate) enum NodeRuntimeMetricsError {
    #[error("Could not write Node.js Language Metrics instrumentation script: {0}")]
    WriteMetricsScript(#[from] std::io::Error),
}

impl From<NodeRuntimeMetricsError> for libcnb::Error<NodeJsEngineBuildpackError> {
    fn from(value: NodeRuntimeMetricsError) -> Self {
        libcnb::Error::BuildpackError(NodeJsEngineBuildpackError::NodeRuntimeMetricsError(value))
    }
}
