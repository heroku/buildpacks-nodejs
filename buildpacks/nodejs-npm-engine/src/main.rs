// cargo-llvm-cov sets the coverage_nightly attribute when instrumenting our code. In that case,
// we enable https://doc.rust-lang.org/beta/unstable-book/language-features/coverage-attribute.html
// to be able selectively opt out of coverage for functions/lines/modules.
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

mod errors;
mod install_npm;
mod node;
mod npm;

use crate::install_npm::{install_npm, NpmInstallError};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use heroku_nodejs_utils::npmjs_org::{
    packument_layer, resolve_package_packument, PackagePackument, PackumentLayerError,
};
use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack, Env};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use serde_json as _;
use std::path::Path;
use std::process::Command;
#[cfg(test)]
use test_support as _;

struct NpmEngineBuildpack;

impl Buildpack for NpmEngineBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NpmEngineBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let package_json_path = context.app_dir.join("package.json");
        if package_json_path.exists() {
            let package_json = PackageJson::read(package_json_path)
                .map_err(NpmEngineBuildpackError::PackageJson)?;
            if package_json
                .engines
                .and_then(|engines| engines.npm)
                .is_some()
            {
                return DetectResultBuilder::pass()
                    .build_plan(
                        BuildPlanBuilder::new()
                            .requires("npm")
                            .requires("node")
                            .provides("npm")
                            .build(),
                    )
                    .build();
            }
        }
        DetectResultBuilder::fail().build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let buildpack_start = print::buildpack(
            context
                .buildpack_descriptor
                .buildpack
                .name
                .as_ref()
                .expect("The buildpack.toml should have a 'name' field set"),
        );

        let env = Env::from_current();
        let requested_npm_version =
            read_requested_npm_version(&context.app_dir.join("package.json"))?;
        let node_version = get_node_version(&env)?;
        let existing_npm_version = get_npm_version(&env)?;

        print::bullet("Determining npm package information");
        let npm_package_packument =
            resolve_requested_npm_packument(&context, &requested_npm_version)?;

        print::bullet("Installing npm");
        if existing_npm_version == npm_package_packument.version {
            print::sub_bullet("Requested npm version is already installed");
        } else {
            install_npm(&context, &npm_package_packument, &node_version)?;
        }

        let npm_version = get_npm_version(&env)?;
        print::sub_bullet(format!(
            "Successfully installed {}",
            style::value(format!("npm@{npm_version}")),
        ));

        print::all_done(&Some(buildpack_start));
        BuildResultBuilder::new().build()
    }

    fn on_error(&self, error: libcnb::Error<Self::Error>) {
        let error_message = errors::on_error(error);
        eprintln!("\n{error_message}");
    }
}

fn read_requested_npm_version(
    package_json_path: &Path,
) -> Result<Requirement, NpmEngineBuildpackError> {
    PackageJson::read(package_json_path)
        .map_err(NpmEngineBuildpackError::PackageJson)
        .and_then(|package_json| {
            package_json
                .engines
                .and_then(|engines| engines.npm)
                .ok_or(NpmEngineBuildpackError::MissingNpmEngineRequirement)
        })
}

fn resolve_requested_npm_packument(
    context: &BuildContext<NpmEngineBuildpack>,
    requested_version: &Requirement,
) -> Result<PackagePackument, libcnb::Error<NpmEngineBuildpackError>> {
    print::sub_bullet(format!(
        "Found {} version {} declared in {}",
        style::value("engines.npm"),
        style::value(requested_version.to_string()),
        style::value("package.json")
    ));

    let npm_packument = packument_layer(context, "npm", NpmEngineBuildpackError::FetchNpmVersions)?;

    let npm_package_packument = resolve_package_packument(&npm_packument, requested_version)
        .ok_or(NpmEngineBuildpackError::NpmVersionResolve(
            requested_version.clone(),
        ))?;

    print::sub_bullet(format!(
        "Resolved npm version {} to {}",
        style::value(requested_version.to_string()),
        style::value(npm_package_packument.version.to_string())
    ));

    Ok(npm_package_packument)
}

fn get_node_version(env: &Env) -> Result<Version, NpmEngineBuildpackError> {
    Command::from(node::Version { env })
        .named_output()
        .map_err(node::VersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| node::VersionError::Parse(stdout, e))
        })
        .map_err(NpmEngineBuildpackError::NodeVersion)
}

fn get_npm_version(env: &Env) -> Result<Version, NpmEngineBuildpackError> {
    Command::from(npm::Version { env })
        .named_output()
        .map_err(npm::VersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| npm::VersionError::Parse(stdout, e))
        })
        .map_err(NpmEngineBuildpackError::NpmVersion)
}

impl From<NpmEngineBuildpackError> for libcnb::Error<NpmEngineBuildpackError> {
    fn from(value: NpmEngineBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

buildpack_main!(NpmEngineBuildpack);

#[derive(Debug)]
pub(crate) enum NpmEngineBuildpackError {
    PackageJson(PackageJsonError),
    MissingNpmEngineRequirement,
    FetchNpmVersions(PackumentLayerError),
    NpmVersionResolve(Requirement),
    NpmInstall(NpmInstallError),
    NodeVersion(node::VersionError),
    NpmVersion(npm::VersionError),
}
