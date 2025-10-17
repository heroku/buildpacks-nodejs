use crate::package_json::{PackageJson, PackageManagerField, PackageManagerFieldPackageManager};
use crate::package_managers::{npm, pnpm, yarn};
use crate::runtimes::nodejs;
use crate::utils::npm_registry::PackagePackument;
use crate::utils::vrs::Requirement;
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::global::print;
use bullet_stream::style;
use libcnb::Env;
use std::path::Path;

// TODO: support `devEngines` field
pub(crate) enum RequestedPackageManager {
    NpmEngine(Requirement),
    PnpmEngine(Requirement),
    YarnEngine(Requirement),
    YarnDefault(Requirement),
    PackageManager(PackageManagerField),
}

impl RequestedPackageManager {
    pub(crate) fn is_npm(&self) -> bool {
        matches!(self, RequestedPackageManager::NpmEngine(_))
            || matches!(
                self,
                RequestedPackageManager::PackageManager(PackageManagerField {
                    name: PackageManagerFieldPackageManager::Npm,
                    ..
                })
            )
    }

    pub(crate) fn is_pnpm(&self) -> bool {
        matches!(self, RequestedPackageManager::PnpmEngine(_))
            || matches!(
                self,
                RequestedPackageManager::PackageManager(PackageManagerField {
                    name: PackageManagerFieldPackageManager::Pnpm,
                    ..
                })
            )
    }

    pub(crate) fn is_yarn(&self) -> bool {
        matches!(self, RequestedPackageManager::YarnEngine(_))
            || matches!(self, RequestedPackageManager::YarnDefault(_))
            || matches!(
                self,
                RequestedPackageManager::PackageManager(PackageManagerField {
                    name: PackageManagerFieldPackageManager::Yarn,
                    ..
                })
            )
    }
}

pub(crate) fn determine_package_manager(
    app_dir: &Path,
    package_json: &PackageJson,
) -> Option<RequestedPackageManager> {
    if let Some(Ok(package_manager_field)) = package_json.package_manager() {
        return Some(RequestedPackageManager::PackageManager(
            package_manager_field,
        ));
    }

    if let Some(Ok(requirement)) = package_json.pnpm_engine() {
        return Some(RequestedPackageManager::PnpmEngine(requirement));
    }
    if let Some(Ok(requirement)) = package_json.yarn_engine() {
        return Some(RequestedPackageManager::YarnEngine(requirement));
    }
    if let Some(Ok(requirement)) = package_json.npm_engine() {
        return Some(RequestedPackageManager::NpmEngine(requirement));
    }

    // fallback to default Yarn if lockfile is detected
    if let Ok(true) = app_dir.join("yarn.lock").try_exists() {
        return Some(RequestedPackageManager::YarnDefault(
            yarn::DEFAULT_YARN_REQUIREMENT.clone(),
        ));
    }

    None
}

pub(crate) fn log_requested_package_manager(requested_package_manager: &RequestedPackageManager) {
    match requested_package_manager {
        RequestedPackageManager::NpmEngine(requirement) => print::sub_bullet(format!(
            "Found {} version {} declared in {}",
            style::value("engines.npm"),
            style::value(requirement.to_string()),
            style::value("package.json")
        )),
        RequestedPackageManager::PnpmEngine(requirement) => print::sub_bullet(format!(
            "Found {} version {} declared in {}",
            style::value("engines.pnpm"),
            style::value(requirement.to_string()),
            style::value("package.json")
        )),
        RequestedPackageManager::YarnEngine(requirement) => print::sub_bullet(format!(
            "Found {} version {} declared in {}",
            style::value("engines.yarn"),
            style::value(requirement.to_string()),
            style::value("package.json")
        )),
        RequestedPackageManager::YarnDefault(requirement) => print::sub_bullet(format!(
            "Found Yarn lockfile, defaulting to {}",
            style::value(requirement.to_string()),
        )),
        RequestedPackageManager::PackageManager(package_manager_field) => {
            print::sub_bullet(format!(
                "Found {} set to {} in {}",
                style::value("packageManager"),
                style::value(package_manager_field.to_string()),
                style::value("package.json")
            ));
        }
    }
}

pub(crate) enum ResolvedPackageManager {
    Npm(Requirement, PackagePackument),
    Pnpm(Requirement, PackagePackument),
    Yarn(Requirement, PackagePackument),
}

pub(crate) fn resolve_package_manager(
    context: &BuildpackBuildContext,
    requested_package_manager: &RequestedPackageManager,
) -> BuildpackResult<ResolvedPackageManager> {
    match requested_package_manager {
        RequestedPackageManager::NpmEngine(requirement) => {
            npm::resolve_npm_package_packument(context, requirement).map(|npm_package_packument| {
                ResolvedPackageManager::Npm(requirement.clone(), npm_package_packument)
            })
        }
        RequestedPackageManager::PnpmEngine(requirement) => {
            pnpm::resolve_pnpm_package_packument(context, requirement).map(
                |pnpm_package_packument| {
                    ResolvedPackageManager::Pnpm(requirement.clone(), pnpm_package_packument)
                },
            )
        }
        RequestedPackageManager::YarnEngine(requirement)
        | RequestedPackageManager::YarnDefault(requirement) => {
            yarn::resolve_yarn_package_packument(context, requirement).map(
                |yarn_package_packument| {
                    ResolvedPackageManager::Yarn(requirement.clone(), yarn_package_packument)
                },
            )
        }
        RequestedPackageManager::PackageManager(package_manager_field) => {
            let requirement = Requirement::parse(&package_manager_field.version.to_string())
                .expect("Exact version string should be a valid requirement range");
            match package_manager_field.name {
                PackageManagerFieldPackageManager::Npm => {
                    npm::resolve_npm_package_packument(context, &requirement).map(
                        |npm_package_packument| {
                            ResolvedPackageManager::Npm(requirement, npm_package_packument)
                        },
                    )
                }
                PackageManagerFieldPackageManager::Pnpm => {
                    pnpm::resolve_pnpm_package_packument(context, &requirement).map(
                        |pnpm_package_packument| {
                            ResolvedPackageManager::Pnpm(requirement, pnpm_package_packument)
                        },
                    )
                }
                PackageManagerFieldPackageManager::Yarn => {
                    yarn::resolve_yarn_package_packument(context, &requirement).map(
                        |yarn_package_packument| {
                            ResolvedPackageManager::Yarn(requirement, yarn_package_packument)
                        },
                    )
                }
            }
        }
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
        ResolvedPackageManager::Pnpm(requested_version, pnpm_package_packument) => {
            print::sub_bullet(format!(
                "Resolved pnpm version {} to {}",
                style::value(requested_version.to_string()),
                style::value(pnpm_package_packument.version.to_string())
            ));
        }
        ResolvedPackageManager::Yarn(requested_version, yarn_package_packument) => {
            print::sub_bullet(format!(
                "Resolved yarn version {} to {}",
                style::value(requested_version.to_string()),
                style::value(yarn_package_packument.version.to_string())
            ));
        }
    }
}

pub(crate) fn install_package_manager(
    context: &BuildpackBuildContext,
    env: &mut Env,
    resolved_package_manager: &ResolvedPackageManager,
) -> BuildpackResult<()> {
    match resolved_package_manager {
        ResolvedPackageManager::Npm(_, npm_package_packument) => {
            print::bullet("Installing npm");
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
        ResolvedPackageManager::Pnpm(_, pnpm_package_packument) => {
            print::bullet("Installing pnpm");
            let pnpm_version = &pnpm_package_packument.version;
            let node_version = nodejs::get_node_version(env)?;
            pnpm::install_pnpm(context, env, pnpm_package_packument, &node_version)?;
            print::sub_bullet(format!(
                "Successfully installed {}",
                style::value(format!("pnpm@{pnpm_version}")),
            ));
            Ok(())
        }
        ResolvedPackageManager::Yarn(_, yarn_package_packument) => {
            print::bullet("Installing Yarn");
            let yarn_version = &yarn_package_packument.version;
            let node_version = nodejs::get_node_version(env)?;
            yarn::install_yarn(context, env, yarn_package_packument, &node_version)?;
            print::sub_bullet(format!(
                "Successfully installed {}",
                style::value(format!("yarn@{yarn_version}")),
            ));
            Ok(())
        }
    }
}
