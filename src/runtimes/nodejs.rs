use crate::utils::download::{
    ChecksumValidator, Downloader, DownloaderError, Extractor, GzipOptions, download_sync,
};
use crate::utils::error_handling::ErrorType::{Internal, UserFacing};
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
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
    CachedLayerDefinition, InvalidMetadataAction, LayerState, RestoredLayerAction,
};
use libcnb::layer_env::Scope;
use libherokubuildpack::inventory::Inventory;
use libherokubuildpack::inventory::artifact::Artifact;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;

pub(crate) static NODEJS_INVENTORY: LazyLock<NodejsInventory> = LazyLock::new(|| {
    toml::from_str(include_str!("../../inventory/nodejs.toml"))
        .expect("Inventory file should be valid")
});

// TODO: Requirement should capture the original requirement string for display purposes but it
//       doesn't (yet), so this wrapper type will have to do for now.
pub(crate) static DEFAULT_NODEJS_REQUIREMENT: LazyLock<DefaultNodeRequirement> =
    LazyLock::new(|| {
        let current_lts = "24.x";
        DefaultNodeRequirement {
            value: current_lts.to_string(),
            requirement: Requirement::parse(current_lts)
                .expect("Default Node.js version should be valid"),
        }
    });

pub(crate) struct DefaultNodeRequirement {
    pub(crate) value: String,
    pub(crate) requirement: Requirement,
}

pub(crate) type NodejsArtifact = Artifact<Version, Sha256, Option<()>>;

pub(crate) type NodejsInventory = Inventory<Version, Sha256, Option<()>>;

pub(crate) fn install(
    context: &BuildpackBuildContext,
    env: &mut Env,
    distribution_artifact: &NodejsArtifact,
) -> BuildpackResult<()> {
    print::bullet("Installing Node.js distribution");

    let new_metadata = NodejsLayerMetadata::from(distribution_artifact.clone());

    let distribution_layer = context.cached_layer(
        // TODO: change this layer name to nodejs_runtime after the package managers are cleaned up
        layer_name!("dist"),
        CachedLayerDefinition {
            build: true,
            launch: true,
            invalid_metadata_action: &|_| InvalidMetadataAction::DeleteLayer,
            restored_layer_action: &|old_metadata: &NodejsLayerMetadata, _| {
                if old_metadata == &new_metadata {
                    RestoredLayerAction::KeepLayer
                } else {
                    RestoredLayerAction::DeleteLayer
                }
            },
        },
    )?;

    let version_tag = format!(
        "{} ({}-{})",
        distribution_artifact.version, distribution_artifact.os, distribution_artifact.arch
    );

    match distribution_layer.state {
        LayerState::Restored { .. } => {
            print::sub_bullet(format!("Reusing Node.js {version_tag}"));
        }
        LayerState::Empty { .. } => {
            download_sync(NodejsArtifactDownloader {
                source: distribution_artifact,
                destination: distribution_layer.path(),
            })
            .map_err(create_downloader_error)?;
            // TODO: this output is meant to match the existing test fixtures but it should be
            //       changed to just output "Installed Node.js version: {}" in the future.
            let version_info = style::value(version_tag);
            print::sub_bullet("Verifying checksum");
            print::sub_bullet(format!("Extracting Node.js {version_info}"));
            print::sub_start_timer(format!("Installing Node.js {version_info}")).done();
            distribution_layer.write_metadata(new_metadata)?;
        }
    }

    env.clone_from(&distribution_layer.read_env()?.apply(Scope::Build, env));

    Ok(())
}

struct NodejsArtifactDownloader<'a> {
    source: &'a NodejsArtifact,
    destination: PathBuf,
}

impl<'a> Downloader<'a> for NodejsArtifactDownloader<'a> {
    fn source_url(&self) -> &str {
        &self.source.url
    }

    fn destination(&self) -> &Path {
        &self.destination
    }

    fn checksum_validator(&self) -> Option<ChecksumValidator<'a>> {
        Some(ChecksumValidator::Sha256(&self.source.checksum.value))
    }

    fn extractor(&self) -> Option<Extractor> {
        Some(Extractor::Gzip(GzipOptions {
            strip_components: 1,
            ..GzipOptions::default()
        }))
    }
}

fn create_downloader_error(error: DownloaderError) -> ErrorMessage {
    match error {
        DownloaderError::Request { url, source } => {
            let nodejs_status_url = style::url("https://status.nodejs.org/");
            let url = style::url(url);
            error_message()
                .id("runtime/nodejs/download/request")
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
                .header("Failed to download Node.js distribution")
                .body(formatdoc! {"
                    A request to download the target Node.js distribution from {url} failed unexpectedly. This error can \
                    occur due to an unstable network connection or an issue with the upstream Node.js distribution \
                    server.

                    Suggestions:
                    - Check the status of {nodejs_status_url} for any reported issues.
                    - Confirm the download url ({url}) works.
                "})
                .debug_info(source.to_string())
                .create()
        }

        DownloaderError::ChecksumMismatch {
            url,
            expected_checksum,
            actual_checksum,
        } => {
            let url = style::url(&url);
            let expected = style::value(&expected_checksum);
            let actual = style::value(&actual_checksum);
            error_message()
                .id("runtime/nodejs/download/checksum")
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header("Node.js distribution checksum verification failed")
                .body(formatdoc! {"
                    An error occurred while verifying the checksum of the Node.js distribution from \
                    {url}. This error can occur due to an issue with the upstream Node.js download server.

                    Checksum:
                    - Expected: {expected}
                    - Actual: {actual}
                "})
                .create()
        }

        DownloaderError::Write {
            url,
            destination,
            source,
        } => {
            let dst_path = file_value(destination);
            let url = style::url(url);
            error_message()
                .id("runtime/nodejs/download/write")
                .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::Yes))
                .header("Failed to copy Node.js distribution contents")
                .body(formatdoc! {"
                    An unexpected I/O error occurred while writing the contents of the Node.js \
                    distribution from {url} to the installation directory at {dst_path}.
                "})
                .debug_info(source.to_string())
                .create()
        }
    }
}

const LAYER_VERSION: &str = "1";

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct NodejsLayerMetadata {
    artifact: NodejsArtifact,
    layer_version: String,
}

impl From<NodejsArtifact> for NodejsLayerMetadata {
    fn from(value: NodejsArtifact) -> Self {
        Self {
            artifact: value,
            layer_version: LAYER_VERSION.to_string(),
        }
    }
}

pub(crate) fn get_node_version(env: &Env) -> BuildpackResult<Version> {
    Command::new("node")
        .envs(env)
        .arg("--version")
        .named_output()
        .try_into()
        .map_err(|e| create_get_node_version_command_error(&e).into())
}

fn create_get_node_version_command_error(error: &VersionCommandError) -> ErrorMessage {
    match error {
        VersionCommandError::Command(e) => error_message()
            .id("runtime/nodejs/get_version")
            .error_type(Internal)
            .header("Failed to determine Node.js version")
            .body(formatdoc! { "
                An unexpected error occurred while attempting to determine the current Node.js version \
                from the system.
            " })
            .debug_info(e.to_string())
            .create(),

        VersionCommandError::Parse(stdout, e) => error_message()
            .id("runtime/nodejs/parse_version")
            .error_type(Internal)
            .header("Failed to parse Node.js version")
            .body(formatdoc! { "
                An unexpected error occurred while parsing Node.js version information from '{stdout}'.
            " })
            .debug_info(e.to_string())
            .create()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;
    use crate::utils::error_handling::test_util::{
        assert_error_snapshot, create_cmd_error, create_reqwest_error,
    };
    use libherokubuildpack::inventory::{
        artifact::{Arch, Os},
        checksum::Checksum,
    };
    use std::str::FromStr;

    fn create_nodejs_artifact(version: &str) -> NodejsArtifact {
        NodejsArtifact {
            version: Version::from_str(version).unwrap(),
            os: Os::Linux,
            arch: Arch::Arm64,
            url: format!(
                "https://nodejs.org/download/release/v{version}/node-v{version}-linux-arm64.tar.gz"
            ),
            checksum: "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                .parse::<Checksum<Sha256>>()
                .unwrap(),
            metadata: None,
        }
    }

    #[test]
    fn metadata_sanity_check() {
        // this is a check to ensure that the same node.js artifact doesn't invalidate the cache
        assert_eq!(
            NodejsLayerMetadata::from(create_nodejs_artifact("22.0.0")),
            NodejsLayerMetadata::from(create_nodejs_artifact("22.0.0"))
        );

        // this is a check to ensure that a different node.js artifact does invalidate the cache
        assert_ne!(
            NodejsLayerMetadata::from(create_nodejs_artifact("22.0.0")),
            NodejsLayerMetadata::from(create_nodejs_artifact("24.0.0")),
        );
    }

    #[test]
    fn metadata_guard() {
        let metadata = NodejsLayerMetadata::from(create_nodejs_artifact("22.0.0"));
        let actual = toml::to_string(&metadata).unwrap();
        let expected = r#"
layer_version = "1"

[artifact]
version = "22.0.0"
os = "linux"
arch = "arm64"
url = "https://nodejs.org/download/release/v22.0.0/node-v22.0.0-linux-arm64.tar.gz"
checksum = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
"#
        .trim();
        assert_eq!(expected, actual.trim());
        let from_toml: NodejsLayerMetadata = toml::from_str(&actual).unwrap();
        assert_eq!(metadata, from_toml);
    }

    #[test]
    fn download_request_error() {
        let url = "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz";
        assert_error_snapshot(&create_downloader_error(DownloaderError::Request {
            url: url.to_string(),
            source: utils::http::Error::Request(url.to_string(), create_reqwest_error()),
        }));
    }

    #[test]
    fn download_write_error() {
        assert_error_snapshot(&create_downloader_error(DownloaderError::Write {
            url: "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
                .into(),
            destination: "/layers/heroku_nodejs/nodejs".into(),
            source: std::io::Error::other("test I/O error"),
        }));
    }

    #[test]
    fn download_checksum_error() {
        assert_error_snapshot(&create_downloader_error(
            DownloaderError::ChecksumMismatch {
                url: "https://nodejs.org/download/release/v23.6.0/node-v23.6.0-linux-arm64.tar.gz"
                    .into(),
                actual_checksum: "e62ff0123a74adfc6903d59a449cbdb0".into(),
                expected_checksum: "d41d8cd98f00b204e9800998ecf8427e".into(),
            },
        ));
    }

    #[test]
    fn test_get_node_version_parse_error() {
        assert_error_snapshot(&create_get_node_version_command_error(
            &VersionCommandError::Parse(
                "not.a.version".into(),
                Version::parse("not.a.version").unwrap_err(),
            ),
        ));
    }

    #[test]
    fn test_get_node_version_command_error() {
        assert_error_snapshot(&create_get_node_version_command_error(
            &VersionCommandError::Command(create_cmd_error("node --version")),
        ));
    }
}
