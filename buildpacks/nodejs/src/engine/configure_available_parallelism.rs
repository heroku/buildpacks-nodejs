use crate::{NodeJsBuildpack, NodeJsBuildpackError};
use heroku_nodejs_utils::available_parallelism::available_parallelism_env;
use libcnb::build::BuildContext;
use libcnb::data::layer_name;
use libcnb::layer::UncachedLayerDefinition;
use libcnb::layer_env::{LayerEnv, ModificationBehavior, Scope};
use libcnb::{additional_buildpack_binary_path, Env};

pub(crate) fn configure_available_parallelism(
    context: &BuildContext<NodeJsBuildpack>,
    env: Env,
) -> Result<Env, libcnb::Error<NodeJsBuildpackError>> {
    let available_parallelism_layer = context.uncached_layer(
        layer_name!("available_parallelism"),
        UncachedLayerDefinition {
            build: true,
            launch: true,
        },
    )?;

    let (available_parallelism_env_key, available_parallelism_env_value) =
        available_parallelism_env();

    // set for the run time env
    available_parallelism_layer.write_exec_d_programs([(
        "available_parallelism",
        additional_buildpack_binary_path!("available_parallelism"),
    )])?;

    // set for the build time env (for webpack plugins or other tools that spin up processes)
    available_parallelism_layer.write_env(LayerEnv::new().chainable_insert(
        Scope::Build,
        ModificationBehavior::Override,
        available_parallelism_env_key,
        available_parallelism_env_value,
    ))?;

    available_parallelism_layer
        .read_env()
        .map(|layer_env| layer_env.apply(Scope::Build, &env))
}
