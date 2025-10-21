use crate::buildpack_config::{BuildpackConfig, ConfigValue, ConfigValueSource};
use crate::package_json::{PackageJson, PackageManagerField, PackageManagerFieldPackageManager};
use crate::package_managers::{npm, pnpm, yarn};
use crate::runtimes::nodejs;
use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use crate::utils::npm_registry::PackagePackument;
use crate::utils::vrs::{Requirement, Version};
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::formatdoc;
use libcnb::Env;
use libcnb::build::BuildResultBuilder;
use libcnb::data::launch::{LaunchBuilder, ProcessBuilder};
use libcnb::data::process_type;
use std::path::{Path, PathBuf};

// TODO: support `devEngines` field
#[derive(Debug, Clone)]
pub(crate) enum RequestedPackageManager {
    BundledNpm,
    NpmEngine(Requirement),
    PnpmEngine(Requirement),
    YarnEngine(Requirement),
    YarnDefault(Requirement),
    YarnVendored(PathBuf),
    PackageManager(PackageManagerField),
}

impl RequestedPackageManager {
    pub(crate) fn is_npm(&self) -> bool {
        matches!(self, RequestedPackageManager::NpmEngine(_))
            || matches!(self, RequestedPackageManager::BundledNpm)
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
            || matches!(self, RequestedPackageManager::YarnVendored(_))
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
) -> RequestedPackageManager {
    // vendored Yarn should take highest priority
    if let Some(Ok(yarnrc)) = yarn::read_yarnrc(app_dir)
        && let Some(yarn_path) = yarnrc.yarn_path()
        && let Ok(true) = yarn_path.try_exists()
    {
        return RequestedPackageManager::YarnVendored(yarn_path);
    }

    // then the package manager field
    if let Some(Ok(package_manager_field)) = package_json.package_manager() {
        return RequestedPackageManager::PackageManager(package_manager_field);
    }

    // then the package manager field
    if let Some(Ok(requirement)) = package_json.pnpm_engine() {
        return RequestedPackageManager::PnpmEngine(requirement);
    }
    if let Some(Ok(requirement)) = package_json.yarn_engine() {
        return RequestedPackageManager::YarnEngine(requirement);
    }
    if let Some(Ok(requirement)) = package_json.npm_engine() {
        return RequestedPackageManager::NpmEngine(requirement);
    }

    // fallback to default Yarn if lockfile is detected
    if let Ok(true) = app_dir.join("yarn.lock").try_exists() {
        return RequestedPackageManager::YarnDefault(yarn::DEFAULT_YARN_REQUIREMENT.clone());
    }

    // default to bundled npm if nothing is requested
    RequestedPackageManager::BundledNpm
}

pub(crate) fn log_requested_package_manager(requested_package_manager: &RequestedPackageManager) {
    // TODO: change this output to something more generic
    if requested_package_manager.is_yarn() {
        print::bullet("Determining Yarn information");
    } else if requested_package_manager.is_pnpm() {
        print::bullet("Determining pnpm package information");
    } else if requested_package_manager.is_npm()
        && !matches!(
            requested_package_manager,
            RequestedPackageManager::BundledNpm
        )
    {
        print::bullet("Determining npm package information");
    }

    match requested_package_manager {
        RequestedPackageManager::BundledNpm => {
            // TODO: this should be reported but will be addressed later
            // E.g.; print::sub_bullet("No npm version requested")
        }
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
        RequestedPackageManager::YarnVendored(yarn_path) => print::sub_bullet(format!(
            "Found {} set to {} in {}",
            style::value("yarnPath"),
            style::value(yarn_path.to_string_lossy()),
            style::value(yarn::Yarnrc::file_name().to_string_lossy())
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
    NpmBundled,
    Pnpm(Requirement, PackagePackument),
    Yarn(Requirement, PackagePackument),
    YarnVendored(PathBuf),
}

pub(crate) fn resolve_package_manager(
    context: &BuildpackBuildContext,
    requested_package_manager: &RequestedPackageManager,
) -> BuildpackResult<ResolvedPackageManager> {
    match requested_package_manager {
        RequestedPackageManager::BundledNpm => Ok(ResolvedPackageManager::NpmBundled),
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
        RequestedPackageManager::YarnVendored(yarn_path) => {
            Ok(ResolvedPackageManager::YarnVendored(yarn_path.clone()))
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
        ResolvedPackageManager::NpmBundled => {
            // TODO: this should be reported but will be addressed later
            // E.g.; print::sub_bullet("Using bundled npm");
        }
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
        ResolvedPackageManager::YarnVendored(yarn_path) => {
            print::sub_bullet(format!(
                "Using vendored yarn at {}",
                style::value(yarn_path.to_string_lossy())
            ));
        }
    }
}

pub(crate) fn install_package_manager(
    context: &BuildpackBuildContext,
    env: &mut Env,
    resolved_package_manager: &ResolvedPackageManager,
) -> BuildpackResult<InstalledPackageManager> {
    match resolved_package_manager {
        ResolvedPackageManager::NpmBundled => {
            // TODO: this needs to be reported but will be addressed later
            // E.g.; print::bullet("Installing npm");
            let npm_version = npm::get_version(env)?;
            // print::sub_bullet(format!(
            //     "Skipping, bundled {} will be used",
            //     style::value(format!("npm@{npm_version}"))
            // ));
            Ok(InstalledPackageManager::Npm(npm_version))
        }
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
            Ok(InstalledPackageManager::Npm(npm_version.clone()))
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
            Ok(InstalledPackageManager::Pnpm)
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
            Ok(InstalledPackageManager::Yarn(yarn_version.clone()))
        }
        ResolvedPackageManager::YarnVendored(yarn_path) => {
            print::bullet("Configuring vendored Yarn");
            print::sub_bullet(format!(
                "Linking {} to vendored {}",
                style::value("yarn"),
                style::value("yarnPath")
            ));
            yarn::link_vendored_yarn(context, env, yarn_path)?;
            let yarn_version = yarn::get_version(env)?;
            print::sub_bullet(format!(
                "Successfully configured {}",
                style::value(format!("yarn@{yarn_version}")),
            ));
            Ok(InstalledPackageManager::Yarn(yarn_version.clone()))
        }
    }
}

pub(crate) enum InstalledPackageManager {
    Npm(Version),
    Pnpm,
    Yarn(Version),
}

pub(crate) fn install_dependencies(
    context: &BuildpackBuildContext,
    env: &Env,
    installed_package_manager: &InstalledPackageManager,
) -> BuildpackResult<()> {
    match installed_package_manager {
        InstalledPackageManager::Npm(version) => {
            npm::install_npm_dependencies(context, env, version)?;
        }
        InstalledPackageManager::Yarn(version) => {
            yarn::install_dependencies(context, env, version)?;
        }
        InstalledPackageManager::Pnpm => {
            unreachable!("Only npm and yarn code should be calling this function")
        }
    }
    Ok(())
}

pub(crate) fn run_build_scripts(
    env: &Env,
    package_manager: &InstalledPackageManager,
    package_json: &PackageJson,
    buildpack_config: &BuildpackConfig,
) -> BuildpackResult<()> {
    print::bullet("Running scripts");
    let build_scripts_enabled = !matches!(
        &buildpack_config.build_scripts_enabled,
        Some(ConfigValue { value: false, .. })
    );

    if [
        "heroku-prebuild",
        "heroku-build",
        "build",
        "heroku-postbuild",
    ]
    .iter()
    .all(|s| package_json.script(s).is_none())
    {
        print::sub_bullet("No build scripts found");
        return Ok(());
    }

    if let Some((prebuild, _)) = package_json.script("heroku-prebuild") {
        if build_scripts_enabled {
            print::sub_stream_cmd(match package_manager {
                InstalledPackageManager::Npm(_) => npm::run_script(&prebuild, env),
                InstalledPackageManager::Yarn(_) => yarn::run_script(&prebuild, env),
                InstalledPackageManager::Pnpm => {
                    unreachable!("Only npm and yarn code should be calling this function")
                }
            })
            .map_err(|e| create_run_script_error_message(&prebuild, &e))?;
        } else {
            print::sub_bullet(format!(
                "Not running {} as it was disabled by a participating buildpack",
                style::value("heroku-prebuild")
            ));
        }
    }

    if let Some((build, _)) = match package_json.script("heroku-build") {
        Some((build, script)) => Some((build, script)),
        None => package_json.script("build"),
    } {
        if build_scripts_enabled {
            print::sub_stream_cmd(match package_manager {
                InstalledPackageManager::Npm(_) => npm::run_script(&build, env),
                InstalledPackageManager::Yarn(_) => yarn::run_script(&build, env),
                InstalledPackageManager::Pnpm => {
                    unreachable!("Only npm and yarn code should be calling this function")
                }
            })
            .map_err(|e| create_run_script_error_message(&build, &e))?;
        } else {
            print::sub_bullet(format!(
                "Not running {} as it was disabled by a participating buildpack",
                style::value(build)
            ));
        }
    }

    if let Some((postbuild, _)) = package_json.script("heroku-postbuild") {
        if build_scripts_enabled {
            print::sub_stream_cmd(match package_manager {
                InstalledPackageManager::Npm(_) => npm::run_script(&postbuild, env),
                InstalledPackageManager::Yarn(_) => yarn::run_script(&postbuild, env),
                InstalledPackageManager::Pnpm => {
                    unreachable!("Only npm and yarn code should be calling this function")
                }
            })
            .map_err(|e| create_run_script_error_message(&postbuild, &e))?;
        } else {
            print::sub_bullet(format!(
                "Not running {} as it was disabled by a participating buildpack",
                style::value("heroku-postbuild")
            ));
        }
    }

    Ok(())
}

fn create_run_script_error_message(script: &str, error: &fun_run::CmdError) -> ErrorMessage {
    let script = style::value(script);
    let script_command = style::command(error.name());
    let package_json = style::value("package.json");
    let heroku_prebuild = style::value("heroku-prebuild");
    let heroku_build = style::value("heroku-build");
    let build = style::value("build");
    let heroku_postbuild = style::value("heroku-postbuild");
    error_message()
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
        .header(format!("Failed to execute build script - {script}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack allows customization of the build process by executing the following scripts \
            if they are defined in {package_json}:
            - {heroku_prebuild}
            - {heroku_build} or {build}
            - {heroku_postbuild}

            An unexpected error occurred while executing {script_command}. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
        "})
        .debug_info(error.to_string())
        .create()
}

pub(crate) fn prune_dev_dependencies(
    env: &Env,
    package_manager: &InstalledPackageManager,
    buildpack_config: &BuildpackConfig,
) -> BuildpackResult<()> {
    print::bullet("Pruning dev dependencies");
    if let Some(ConfigValue {
        value: false,
        source,
    }) = &buildpack_config.prune_dev_dependencies
    {
        // TODO: revisit this output as it could be simplified.
        match source {
            ConfigValueSource::Buildplan(_) => {
                print::sub_bullet("Skipping as pruning was disabled by a participating buildpack");
            }
            ConfigValueSource::ProjectToml => {
                print::sub_bullet("Skipping as pruning was disabled in project.toml");
            }
        }
        return Ok(());
    }

    print::sub_stream_cmd(match package_manager {
        InstalledPackageManager::Npm(_) => npm::prune_dev_dependencies(env),
        InstalledPackageManager::Yarn(yarn_version) => {
            yarn::prune_dev_dependencies(env, yarn_version)?
        }
        InstalledPackageManager::Pnpm => {
            unreachable!("Only npm and Yarn code should be calling this function")
        }
    })
    .map_err(|e| create_prune_dev_dependencies_error_message(&e))?;

    Ok(())
}

fn create_prune_dev_dependencies_error_message(error: &fun_run::CmdError) -> ErrorMessage {
    let prune_command = style::value(error.name());
    error_message()
        .error_type(ErrorType::UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header("Failed to prune dev dependencies")
        .body(formatdoc! { "
            The Heroku Node.js buildpack uses the command {prune_command} to remove your dev dependencies from the production \
            environment. This command failed and the buildpack cannot continue. See the log output above for more information.

            Suggestions:
            - Ensure that this command runs locally without error (exit status = 0).
        " })
        .debug_info(error.to_string())
        .create()
}

pub(crate) fn configure_default_processes(
    context: &BuildpackBuildContext,
    build_result_builder: BuildResultBuilder,
    package_json: &PackageJson,
    installed_package_manager: &InstalledPackageManager,
) -> BuildResultBuilder {
    if let Ok(true) = context.app_dir.join("Procfile").try_exists() {
        print::bullet("Configuring default processes");
        print::sub_bullet("Skipping default web process (Procfile detected)");
        build_result_builder
    } else if package_json.script("start").is_some() {
        match installed_package_manager {
            InstalledPackageManager::Npm(_) => {
                print::bullet("Configuring default processes");
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
            }
            _ => build_result_builder,
        }
    } else {
        print::bullet("Configuring default processes");
        print::sub_bullet("Skipping default web process (no start script defined)");
        build_result_builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_cmd_error};

    #[test]
    fn run_script_error_message() {
        assert_error_snapshot(&create_run_script_error_message(
            "build",
            &create_cmd_error("<package_manager> run build"),
        ));
    }

    #[test]
    fn prune_dev_dependencies_error_message() {
        assert_error_snapshot(&create_prune_dev_dependencies_error_message(
            &create_cmd_error("<package_manager> prune"),
        ));
    }
}
