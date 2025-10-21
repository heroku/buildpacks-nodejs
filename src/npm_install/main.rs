// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use crate::utils::error_handling::ErrorMessage;
use crate::utils::package_json::{PackageJson, PackageJsonError};
use crate::{BuildpackBuildContext, BuildpackError, BuildpackResult, NodeJsBuildpackError};
use bullet_stream::global::print;
use bullet_stream::style;
use libcnb::Env;
use libcnb::build::BuildResultBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;

pub(crate) fn build(
    context: &BuildpackBuildContext,
    env: Env,
    build_result_builder: BuildResultBuilder,
) -> BuildpackResult<(Env, BuildResultBuilder)> {
    let app_dir = &context.app_dir;
    let package_json = PackageJson::read(app_dir.join("package.json"))
        .map_err(NpmInstallBuildpackError::PackageJson)?;

    print::bullet("Configuring default processes");
    let build_result_builder =
        configure_default_processes(context, build_result_builder, &package_json);

    Ok((env, build_result_builder))
}

pub(crate) fn on_error(error: NpmInstallBuildpackError) -> ErrorMessage {
    super::errors::on_npm_install_buildpack_error(error)
}

fn configure_default_processes(
    context: &BuildpackBuildContext,
    build_result_builder: BuildResultBuilder,
    package_json: &PackageJson,
) -> BuildResultBuilder {
    if context.app_dir.join("Procfile").exists() {
        print::sub_bullet("Skipping default web process (Procfile detected)");
        build_result_builder
    } else if package_json.has_start_script() {
        print::sub_bullet(format!(
            "Adding default web process for {}",
            style::value("npm start")
        ));
        build_result_builder.launch(
            LaunchBuilder::new()
                .process(
                    ProcessBuilder::new(process_type!("web"), ["npm", "start"])
                        .default(true)
                        .build(),
                )
                .build(),
        )
    } else {
        print::sub_bullet("Skipping default web process (no start script defined)");
        build_result_builder
    }
}

#[derive(Debug)]
pub(crate) enum NpmInstallBuildpackError {
    PackageJson(PackageJsonError),
}

impl From<NpmInstallBuildpackError> for BuildpackError {
    fn from(value: NpmInstallBuildpackError) -> Self {
        NodeJsBuildpackError::NpmInstall(value).into()
    }
}
