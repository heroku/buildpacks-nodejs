// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::buildpack_config::{BuildpackConfig, ConfigValue, ConfigValueSource};
use crate::utils::error_handling::ErrorMessage;
use crate::utils::package_json::{PackageJson, PackageJsonError};
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use indoc::indoc;
use libcnb::Env;
use libcnb::build::BuildResultBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;

#[allow(clippy::too_many_lines)]
pub(crate) fn build(
    context: &BuildpackBuildContext,
    env: Env,
    mut build_result_builder: BuildResultBuilder,
    buildpack_config: &BuildpackConfig,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    let pkg_json_file = context.app_dir.join("package.json");
    let pkg_json =
        PackageJson::read(&pkg_json_file).map_err(PnpmInstallBuildpackError::PackageJson)?;

    if context.app_dir.join("Procfile").exists() {
        print::bullet("Skipping default web process (Procfile detected)");
    } else if pkg_json.has_start_script() {
        build_result_builder = build_result_builder.launch(
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), ["pnpm", "start"])
                        .default(true)
                        .build(),
                )
                .build(),
        );
    }

    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            source: ConfigValueSource::ProjectToml,
            ..
        })
    ) {
        print::warning(indoc! { "
            Warning: Experimental configuration `com.heroku.buildpacks.nodejs.actions.prune_dev_dependencies` \
            found in `project.toml`. This feature may change unexpectedly in the future.
        " });
    }

    if matches!(
        buildpack_config.prune_dev_dependencies,
        Some(ConfigValue {
            source: ConfigValueSource::Buildplan(_),
            ..
        })
    ) {
        print::warning(indoc! { "
            Warning: Experimental configuration `node_build_scripts.metadata.skip_pruning` was added \
            to the buildplan by a later buildpack. This feature may change unexpectedly in the future.
        " });
    }

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: PnpmInstallBuildpackError) -> ErrorMessage {
    super::errors::on_pnpm_install_buildpack_error(error)
}

#[derive(Debug)]
pub(crate) enum PnpmInstallBuildpackError {
    PackageJson(PackageJsonError),
}

impl From<PnpmInstallBuildpackError> for BuildpackError {
    fn from(e: PnpmInstallBuildpackError) -> Self {
        NodeJsBuildpackError::PnpmInstall(e).into()
    }
}
