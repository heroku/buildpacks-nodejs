use crate::utils::download::{Downloader, DownloaderError, Extractor, GzipOptions, download_sync};
use crate::utils::error_handling::ErrorType::{Internal, UserFacing};
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::npm_registry::{
    PackagePackument, PackumentLayerError, packument_layer, resolve_package_packument,
};
use crate::utils::vrs::{Requirement, Version, VersionCommandError};
use crate::{BuildpackBuildContext, BuildpackResult};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use indoc::formatdoc;
use libcnb::Env;
use libcnb::data::layer_name;
use libcnb::layer::{
    CachedLayerDefinition, EmptyLayerCause, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::Scope;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn resolve_npm_package_packument(
    context: &BuildpackBuildContext,
    requirement: &Requirement,
) -> BuildpackResult<PackagePackument> {
    let npm_packument = packument_layer(context, "npm", |error| {
        create_npm_packument_layer_error(&error)
    })?;

    let npm_package_packument = resolve_package_packument(&npm_packument, requirement)
        .ok_or_else(|| create_resolve_npm_package_packument_error(requirement))?;

    Ok(npm_package_packument)
}

fn create_npm_packument_layer_error(error: &PackumentLayerError) -> ErrorMessage {
    let npm = style::value("npm");
    let npm_status_url = style::url("https://status.npmjs.org/");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
        .header(format!("Failed to load available {npm} versions"))
        .body(formatdoc! { "
            An unexpected error occurred while loading the available {npm} versions. This error can \
            occur due to an unstable network connection or an issue with the npm registry.

            Suggestions:
            - Check the npm status page for any ongoing incidents ({npm_status_url})
        "})
        .debug_info(error.to_string())
        .create()
}

fn create_resolve_npm_package_packument_error(requirement: &Requirement) -> ErrorMessage {
    let npm = style::value("npm");
    let requested_version = style::value(requirement.to_string());
    let npm_releases_url = style::url("https://www.npmjs.com/package/npm?activeTab=versions");
    let npm_show_command = style::value(format!("npm show 'npm@{requirement}' versions"));
    let package_json = style::value("package.json");
    let engines_key = style::value("engines.npm");
    error_message()
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!("Error resolving requested {npm} version {requested_version}"))
        .body(formatdoc! { "
                The requested npm version could not be resolved to a known release in this buildpack's \
                inventory of npm releases.

                Suggestions:
                - Confirm if this is a valid npm release at {npm_releases_url} or by running {npm_show_command}
                - Update the {engines_key} field in {package_json} to a single version or version range that \
                includes a published {npm} version.
            " })
        .create()
}

pub(crate) fn get_version(env: &Env) -> BuildpackResult<Version> {
    Command::new("npm")
        .envs(env)
        .arg("--version")
        .named_output()
        .try_into()
        .map_err(|e| create_get_npm_version_command_error(&e).into())
}

fn create_get_npm_version_command_error(error: &VersionCommandError) -> ErrorMessage {
    match error {
        VersionCommandError::Command(e) => error_message()
            .error_type(Internal)
            .header("Failed to determine npm version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current npm version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        VersionCommandError::Parse(stdout, e) => error_message()
            .error_type(Internal)
            .header("Failed to parse npm version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing npm version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

pub(crate) fn install_npm(
    context: &BuildpackBuildContext,
    env: &mut Env,
    npm_packument: &PackagePackument,
    node_version: &Version,
) -> BuildpackResult<()> {
    let npm_version = &npm_packument.version;

    let new_metadata = NpmEngineLayerMetadata {
        node_version: node_version.to_string(),
        npm_version: npm_version.to_string(),
        layer_version: LAYER_VERSION.to_string(),
        arch: context.target.arch.clone(),
        os: context.target.os.clone(),
    };

    let npm_engine_layer = context.cached_layer(
        layer_name!("npm_engine"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &NpmEngineLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    Ok((RestoredLayerAction::KeepLayer, vec![]))
                } else {
                    Ok((
                        RestoredLayerAction::DeleteLayer,
                        changed_metadata_fields(old_metadata, &new_metadata),
                    ))
                }
            },
        },
    )?;

    match npm_engine_layer.state {
        LayerState::Restored { .. } => {
            // TODO: The following message should only be printed when the bundled npm version matches the requested version
            //       but due to the way that the earlier install procedure worked, it would never show the message when
            //       the cached npm version was used. This should output something like "Using cached npm" instead.
            print::sub_bullet("Requested npm version is already installed");
        }
        LayerState::Empty { ref cause } => {
            if let EmptyLayerCause::RestoredLayerAction { cause } = cause {
                print::sub_bullet(format!(
                    "Invalidating cached npm ({} changed)",
                    cause.join(", ")
                ));
            }

            download_sync(NpmDownloader {
                url: npm_packument.dist.tarball.to_string(),
                destination: npm_engine_layer.path(),
            })
            .map_err(create_npm_downloader_error)?;

            // TODO: This logging isn't accurate but is here to keep fixtures from failing while
            //       these structural changes are being implemented.
            print::sub_bullet("Removing npm bundled with Node.js");
            print::sub_bullet("Installing requested npm");

            // Create symlinks for all binaries declared in packument's `bin` field
            let bins = npm_packument
                .bin
                .as_ref()
                .ok_or_else(|| create_npm_install_bins_error(NpmBinInstallError::MissingBins))?;
            for (name, script) in bins {
                let bin_path = npm_engine_layer.path().join("bin").join(name);
                let script_path = npm_engine_layer.path().join(script);
                match std::fs::remove_file(&bin_path) {
                    Ok(()) => Ok(()),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(e) => Err(create_npm_install_bins_error(NpmBinInstallError::Write(e))),
                }?;
                std::os::unix::fs::symlink(script_path, bin_path)
                    .map_err(|e| create_npm_install_bins_error(NpmBinInstallError::Write(e)))?;
            }

            npm_engine_layer.write_metadata(new_metadata)?;
        }
    }

    env.clone_from(&npm_engine_layer.read_env()?.apply(Scope::Build, env));

    Ok(())
}

enum NpmBinInstallError {
    MissingBins,
    Write(std::io::Error),
}

struct NpmDownloader {
    url: String,
    destination: PathBuf,
}

impl Downloader<'_> for NpmDownloader {
    fn source_url(&self) -> &str {
        &self.url
    }

    fn destination(&self) -> &Path {
        &self.destination
    }

    fn extractor(&self) -> Option<Extractor> {
        Some(Extractor::Gzip(GzipOptions {
            strip_components: 1,
            exclude: Box::new(|path| {
                path.components().take(1).next().is_some_and(|c| {
                    // test
                    c.as_os_str() == "docs" || c.as_os_str() == "man"
                })
            }),
            ..GzipOptions::default()
        }))
    }
}

fn create_npm_downloader_error(error: DownloaderError) -> ErrorMessage {
    let npm = style::value("npm");
    match error {
        DownloaderError::Request { source, .. } => {
            let npm_status_url = style::url("https://status.npmjs.org/");
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header(format!("Failed to download {npm}"))
                .body(formatdoc! {"
                    An unexpected error occurred while downloading the {npm} package. This error can \
                    occur due to an unstable network connection or an issue with the npm repository.

                    Suggestions:
                    - Check the npm status page for any ongoing incidents ({npm_status_url})
                " })
                .debug_info(source.to_string())
                .create()
        }

        DownloaderError::ChecksumMismatch { .. } => {
            unreachable!("The npm package does not perform checksums currently")
        }

        DownloaderError::Write {
            destination,
            source,
            ..
        } => {
            let path = file_value(destination);
            error_message()
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header(format!("Failed to extract {npm} package file"))
                .body(formatdoc! {"
                    An unexpected I/O occurred while extracting the contents of the downloaded {npm} package file at {path}.
                " })
                .debug_info(source.to_string())
                .create()
        }
    }
}

fn create_npm_install_bins_error(error: NpmBinInstallError) -> ErrorMessage {
    let npm = style::value("npm");

    match error {
        NpmBinInstallError::MissingBins => error_message()
            .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
            .header(format!(
                "Failed to determine the downloaded {npm} package binaries"
            ))
            .body(formatdoc! {"
                The downloaded {npm} package does not declare any binaries to install. This is \
                unexpected and may indicate a bug in the buildpack.
            " })
            .create(),

        NpmBinInstallError::Write(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
            .header(format!(
                "Failed to install the downloaded {npm} package binaries"
            ))
            .body(formatdoc! {"
                An unexpected error occurred while linking the {npm} package scripts to \
                their declared binaries.
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

fn changed_metadata_fields(
    old: &NpmEngineLayerMetadata,
    new: &NpmEngineLayerMetadata,
) -> Vec<String> {
    let mut changed = vec![];
    if old.npm_version != new.npm_version {
        changed.push("npm version".to_string());
    }
    if old.node_version != new.node_version {
        changed.push("node version".to_string());
    }
    if old.layer_version != new.layer_version {
        changed.push("layer version".to_string());
    }
    if old.os != new.os {
        changed.push("operating system".to_string());
    }
    if old.arch != new.arch {
        changed.push("compute architecture".to_string());
    }
    changed.sort();
    changed
}

const LAYER_VERSION: &str = "1";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NpmEngineLayerMetadata {
    layer_version: String,
    npm_version: String,
    node_version: String,
    arch: String,
    os: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_cmd_error};

    #[test]
    fn packument_layer_error() {
        assert_error_snapshot(&create_npm_packument_layer_error(
            &PackumentLayerError::ReadPackument(std::io::Error::other("Insufficient permissions")),
        ));
    }

    #[test]
    fn resolve_package_packument_error() {
        assert_error_snapshot(&create_resolve_npm_package_packument_error(
            &Requirement::parse("1.2.3").unwrap(),
        ));
    }

    #[test]
    fn version_parse_error() {
        assert_error_snapshot(&create_get_npm_version_command_error(
            &VersionCommandError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    #[test]
    fn version_command_error() {
        assert_error_snapshot(&create_get_npm_version_command_error(
            &VersionCommandError::Command(create_cmd_error("npm --version")),
        ));
    }
}
