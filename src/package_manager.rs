use crate::package_json::PackageJson;
use crate::utils::vrs::Requirement;
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
