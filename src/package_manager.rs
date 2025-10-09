use crate::package_json::PackageJson;
use crate::package_managers::npm;
use crate::utils::npm_registry::PackagePackument;
use crate::utils::vrs::Requirement;
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::global::print;
use bullet_stream::style;

pub(crate) enum RequestedPackageManager {
    Npm(RequestedNpm),
}

// TODO: support `packageManager` field
// TODO: support `devEngines` field
#[allow(dead_code)]
pub(crate) enum RequestedNpm {
    NpmEngine(Requirement),
}

pub(crate) fn determine_package_manager(
    package_json: &PackageJson,
) -> Option<RequestedPackageManager> {
    // TODO: this will eventually need to check for lockfiles to determine the package manager
    //       but due to how this is currently being called, only npm can be returned
    package_json
        .npm_engine()
        .map(RequestedNpm::NpmEngine)
        .map(RequestedPackageManager::Npm)
}

pub(crate) fn log_requested_package_manager(requested_package_manager: &RequestedPackageManager) {
    match requested_package_manager {
        RequestedPackageManager::Npm(requested_npm) => match requested_npm {
            RequestedNpm::NpmEngine(requirement) => print::sub_bullet(format!(
                "Found {} version {} declared in {}",
                style::value("engines.npm"),
                style::value(requirement.to_string()),
                style::value("package.json")
            )),
        },
    }
}

pub(crate) enum ResolvedPackageManager {
    Npm(Requirement, PackagePackument),
}

pub(crate) fn resolve_package_manager(
    context: &BuildpackBuildContext,
    requested_package_manager: &RequestedPackageManager,
) -> BuildpackResult<ResolvedPackageManager> {
    match requested_package_manager {
        RequestedPackageManager::Npm(requested_npm) => match requested_npm {
            RequestedNpm::NpmEngine(requirement) => {
                npm::resolve_npm_package_packument(context, requirement).map(
                    |npm_package_packument| {
                        ResolvedPackageManager::Npm(requirement.clone(), npm_package_packument)
                    },
                )
            }
        },
    }
}

pub(crate) fn log_resolved_package_manager(resolved_package_manager: &ResolvedPackageManager) {
    match resolved_package_manager {
        ResolvedPackageManager::Npm(requested_version, npm_package_packument) => {
            print::sub_bullet(format!(
                "Resolved npm version {} to {}",
                style::value(requested_version.to_string()),
                style::value(npm_package_packument.version.to_string())
            ));
        }
    }
}
