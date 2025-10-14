use crate::package_json::{PackageJson, PackageManagerField, PackageManagerFieldPackageManager};
use crate::package_managers::npm;
use crate::runtimes::nodejs;
use crate::utils::npm_registry::PackagePackument;
use crate::utils::vrs::Requirement;
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::global::print;
use bullet_stream::style;
use libcnb::Env;

pub(crate) enum RequestedPackageManager {
    Npm(RequestedNpm),
}

// TODO: support `devEngines` field
pub(crate) enum RequestedNpm {
    NpmEngine(Requirement),
    PackageManager(PackageManagerField),
}

pub(crate) fn determine_package_manager(
    package_json: &PackageJson,
) -> Option<RequestedPackageManager> {
    // TODO: this will eventually need to check for lockfiles to determine the package manager
    //       but due to how this is currently being called, only npm can be returned
    if let Some(Ok(package_manager_field)) = package_json.package_manager()
        && package_manager_field.name == PackageManagerFieldPackageManager::Npm
    {
        Some(RequestedPackageManager::Npm(RequestedNpm::PackageManager(
            package_manager_field,
        )))
    } else {
        package_json
            .npm_engine()
            .map(RequestedNpm::NpmEngine)
            .map(RequestedPackageManager::Npm)
    }
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
            RequestedNpm::PackageManager(package_manager_field) => {
                let package_manager = style::value("packageManager");
                let package_json = style::value("package.json");
                let value = style::value(package_manager_field.to_string());
                print::sub_bullet(format!(
                    "Found {package_manager} set to {value} in {package_json}"
                ));
            }
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
            RequestedNpm::PackageManager(package_manager_field) => {
                let requirement = Requirement::parse(&package_manager_field.version.to_string())
                    .expect("Exact version string should be a valid requirement range");
                npm::resolve_npm_package_packument(context, &requirement).map(
                    |npm_package_packument| {
                        ResolvedPackageManager::Npm(requirement, npm_package_packument)
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

pub(crate) fn install_package_manager(
    context: &BuildpackBuildContext,
    env: &mut Env,
    resolved_package_manager: &ResolvedPackageManager,
) -> BuildpackResult<()> {
    print::bullet("Installing npm");
    match resolved_package_manager {
        ResolvedPackageManager::Npm(_, npm_package_packument) => {
            let npm_version = &npm_package_packument.version;
            let node_version = nodejs::get_node_version(env)?;
            let bundled_npm_version = npm::get_version(env)?;
            if bundled_npm_version == npm_package_packument.version {
                print::sub_bullet("Requested npm version is already installed");
            } else {
                npm::install_npm(context, env, npm_package_packument, &node_version)?;
            }
            print::sub_bullet(format!(
                "Successfully installed {}",
                style::value(format!("npm@{npm_version}")),
            ));
            Ok(())
        }
    }
}
