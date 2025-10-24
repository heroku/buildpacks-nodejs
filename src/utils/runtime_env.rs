use crate::{BuildpackBuildContext, BuildpackResult};
use libcnb::data::layer::LayerName;
use libcnb::layer::UncachedLayerDefinition;
use std::path::PathBuf;

pub(crate) fn register_execd_script(
    context: &BuildpackBuildContext,
    layer_name: LayerName,
    script: PathBuf,
) -> BuildpackResult<()> {
    let program_name = layer_name.to_string();
    let layer = context.uncached_layer(
        layer_name,
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;
    layer.write_exec_d_programs([(program_name, script)])?;
    Ok(())
}
