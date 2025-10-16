use crate::utils::npm_registry::{PackagePackument, packument_layer, resolve_package_packument};
use crate::utils::vrs::{Requirement, Version};
use crate::{BuildpackBuildContext, BuildpackResult, utils};
use libcnb::Env;
use libcnb::data::layer_name;

pub(crate) fn resolve_pnpm_package_packument(
    context: &BuildpackBuildContext,
    requirement: &Requirement,
) -> BuildpackResult<PackagePackument> {
    resolve_package_packument(
        &packument_layer(layer_name!("pnpm_packument"), context, "pnpm")?,
        requirement,
    )
    .map_err(Into::into)
}

pub(crate) fn install_pnpm(
    context: &BuildpackBuildContext,
    env: &mut Env,
    pnpm_packument: &PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    utils::npm_registry::install_package_layer(
        layer_name!("pnpm"),
        context,
        env,
        pnpm_packument,
        node_version,
    )
    .map_err(Into::into)
}
