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

pub(crate) fn node_gyp_env() -> Vec<(String, String)> {
    vec![
        // If this is set to a non-empty string, Python won’t try to write .pyc files on the import of source modules.
        // see https://docs.python.org/3/using/cmdline.html#envvar-PYTHONDONTWRITEBYTECODE
        //
        // This is used to prevent node-gyp from generating files which invalidate runtime layers whenever
        // native modules are compiled as `node-gyp` uses Python as part of the compilation process.
        ("PYTHONDONTWRITEBYTECODE".to_string(), "1".to_string()),
    ]
}
