use crate::{NodeJsEngineBuildpack, NodeJsEngineBuildpackError};
use libcnb::build::BuildContext;
use libcnb::data::layer_content_metadata::LayerTypes;
use libcnb::generic::GenericMetadata;
use libcnb::layer::{Layer, LayerResult, LayerResultBuilder};
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use std::fs;
use std::path::Path;
use thiserror::Error;

pub(crate) struct NodeRuntimeMetricsLayer;

impl Layer for NodeRuntimeMetricsLayer {
    type Buildpack = NodeJsEngineBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: false,
            launch: true,
            cache: false,
        }
    }

    fn create(
        &mut self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, NodeJsEngineBuildpackError> {
        let metrics_script = layer_path.join("metrics_collector.cjs");

        fs::write(
            &metrics_script,
            include_bytes!("../../node_runtime_metrics/metrics_collector.cjs"),
        )
        .map_err(NodeRuntimeMetricsError::WriteMetricsScript)?;

        LayerResultBuilder::new(GenericMetadata::default())
            .env(
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
            )
            .build()
    }
}

#[derive(Debug, Error)]
pub(crate) enum NodeRuntimeMetricsError {
    #[error("Could not write Node.js Language Metrics instrumentation script: {0}")]
    WriteMetricsScript(#[from] std::io::Error),
}
