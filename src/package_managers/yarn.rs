use crate::utils::npm_registry::{PackagePackument, packument_layer, resolve_package_packument};
use crate::utils::vrs::{Requirement, Version};
use crate::{BuildpackBuildContext, BuildpackResult, utils};
use libcnb::Env;
use libcnb::data::layer_name;
use std::sync::LazyLock;

static YARN_BERRY_RANGE: LazyLock<Requirement> = LazyLock::new(|| {
    Requirement::parse(">=2").expect("Yarn berry requirement range should be valid")
});

pub(crate) fn resolve_yarn_package_packument(
    context: &BuildpackBuildContext,
    requirement: &Requirement,
) -> BuildpackResult<PackagePackument> {
    let (yarn_layer_name, yarn_package_name) = if requirement.allows_any(&YARN_BERRY_RANGE) {
        (
            layer_name!("yarnpkg_cli-dist_packument"),
            "@yarnpkg/cli-dist",
        )
    } else {
        (layer_name!("yarn_packument"), "yarn")
    };
    resolve_package_packument(
        &packument_layer(yarn_layer_name, context, yarn_package_name)?,
        requirement,
    )
    .map_err(Into::into)
}

pub(crate) fn install_yarn(
    context: &BuildpackBuildContext,
    env: &mut Env,
    yarn_packument: &PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    utils::npm_registry::install_package_layer(
        layer_name!("yarn"),
        context,
        env,
        yarn_packument,
        node_version,
    )
    .map_err(Into::into)
}
