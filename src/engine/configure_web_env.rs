use crate::{NodejsBuildpack, NodejsBuildpackError};
use libcnb::additional_buildpack_binary_path;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::UncachedLayerDefinition;

pub(crate) fn configure_web_env(
    context: &BuildContext<NodejsBuildpack>,
) -> Result<(), libcnb::Error<NodejsBuildpackError>> {
    let web_env_layer = context.uncached_layer(
        layer_name!("web_env"),
        UncachedLayerDefinition {
            build: false,
            launch: true,
        },
    )?;

    web_env_layer
        .write_exec_d_programs([("web_env", additional_buildpack_binary_path!("web_env"))])?;

    Ok(())
}
