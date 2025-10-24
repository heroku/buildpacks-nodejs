use crate::{BuildpackBuildContext, BuildpackError};
use libcnb::Env;
use libcnb::data::layer::LayerName;
use libcnb::layer::UncachedLayerDefinition;
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use std::ffi::OsString;

pub(crate) fn set_default_env_var(
    context: &BuildpackBuildContext,
    env: &mut Env,
    name: impl Into<OsString>,
    value: impl Into<OsString>,
) -> Result<(), BuildpackError> {
    let name = name.into();
    let value = value.into();
    let layer_name = format!("build_default_{}", name.to_string_lossy().to_lowercase())
        .parse::<LayerName>()
        .expect("Layer name should be valid");

    let layer = context.uncached_layer(
        layer_name,
        UncachedLayerDefinition {
            build: true,
            launch: false,
        },
    )?;

    let mut layer_env = LayerEnv::new();
    layer_env.insert(Scope::Build, ModificationBehavior::Default, name, value);

    layer.write_env(layer_env)?;

    env.clone_from(&layer.read_env()?.apply(Scope::Build, env));

    Ok(())
}
