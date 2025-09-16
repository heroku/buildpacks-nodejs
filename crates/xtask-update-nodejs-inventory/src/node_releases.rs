use crate::SupportedNodeReleasePlatform;
use crate::trusted_release_keys::NodeReleaseKeys;
use libherokubuildpack::inventory::Inventory;
use libherokubuildpack::inventory::artifact::Artifact;
use libherokubuildpack::inventory::checksum::Checksum;
use node_semver::Version;
use reqwest::{IntoUrl, Url};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use sequoia_openpgp::crypto::HashAlgorithm;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::parse::stream::DetachedVerifierBuilder;
use sequoia_openpgp::policy::StandardPolicy;
use serde::Deserialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;

static BASE_NODEJS_RELEASE_URL: LazyLock<Url> = LazyLock::new(|| {
    Url::parse("https://nodejs.org/download/release/").expect("base url should be valid")
});

static STARTING_NODE_VERSION: LazyLock<Version> =
    LazyLock::new(|| Version::parse("0.8.6").expect("Starting Node.js version should be valid"));

pub(super) fn from_inventory(inventory_path: &Path) -> Vec<NodeArtifact> {
    std::fs::read_to_string(inventory_path)
        .expect("Failed to read inventory file")
        .parse::<Inventory<Version, Sha256, Option<()>>>()
        .unwrap_or(Inventory::default())
        .artifacts
}

pub(super) async fn from_upstream(
    inventory: &[NodeArtifact],
    supported_node_release_platforms: &[&SupportedNodeReleasePlatform],
    node_release_keys: NodeReleaseKeys,
) -> Vec<NodeArtifact> {
    let mut upstream_artifacts = vec![];
    'release: for node_release in get_release_index().await {
        if node_release.version < *STARTING_NODE_VERSION {
            eprintln!(
                "Skipping Node.js version {} as the earliest version supported is {}",
                node_release.version, *STARTING_NODE_VERSION
            );
            continue 'release;
        }

        // This is used to avoid downloading the checksums multiple times for each supported platform
        let mut downloaded_release_checksums = None;

        'platform: for supported_platform in supported_node_release_platforms {
            if !node_release
                .files
                .contains(&supported_platform.release_file())
            {
                eprintln!(
                    "Skipping Node.js version {} ({}) as there is no release file provided",
                    node_release.version,
                    supported_platform.release_file()
                );
                continue 'platform;
            }

            if let Some(artifact) = inventory.iter().find(|artifact| {
                artifact.arch == supported_platform.arch()
                    && artifact.os == supported_platform.os()
                    && artifact.version == node_release.version
            }) {
                upstream_artifacts.push(artifact.clone());
                continue 'platform;
            }

            if downloaded_release_checksums.is_none() {
                eprintln!(
                    "Downloading checksums for Node.js version {}",
                    node_release.version
                );
                downloaded_release_checksums =
                    Some(get_release_checksums(&node_release, node_release_keys.clone()).await);
            }

            let release_checksums = downloaded_release_checksums.as_ref().unwrap_or_else(|| {
                panic!(
                    "Should have downloaded checksums for Node.js version {}",
                    node_release.version
                )
            });

            let checksum = release_checksums
                .get(&node_release.binary_file_name(supported_platform))
                .unwrap_or_else(|| {
                    panic!(
                        "Should have found checksum for Node.js version {} ({})",
                        node_release.version,
                        supported_platform.release_file()
                    )
                });

            upstream_artifacts.push(NodeArtifact {
                url: node_release.binary_url(supported_platform).to_string(),
                version: node_release.version.clone(),
                checksum: format!("sha256:{checksum}")
                    .parse::<Checksum<Sha256>>()
                    .unwrap_or_else(|_| panic!("Failed to parse checksum 'sha256:{checksum}'")),
                arch: supported_platform.arch(),
                os: supported_platform.os(),
                metadata: None,
            });
        }
    }

    upstream_artifacts
}

async fn get_release_index() -> Vec<NodeRelease> {
    let release_index_file = download_file(
        BASE_NODEJS_RELEASE_URL
            .join("index.json")
            .expect("Release index URL should be valid"),
    )
    .await;

    let release_index_contents =
        std::fs::read_to_string(&release_index_file).expect("Failed to read release index file");

    serde_json::from_str::<Vec<NodeRelease>>(&release_index_contents)
        .expect("Failed to parse release index file")
}

async fn get_release_checksums(
    node_release: &NodeRelease,
    node_release_keys: NodeReleaseKeys,
) -> HashMap<String, String> {
    let shasums = download_file(node_release.shasums_url()).await;
    let shasums_sig = download_file(node_release.shasums_sig_url()).await;

    eprintln!("Verifying integrity of checksums...");
    let mut standard_policy = StandardPolicy::new();

    // Some of the trusted release keys are signed with SHA1
    // https://gitlab.com/sequoia-pgp/sequoia/-/issues/595
    standard_policy.accept_hash(HashAlgorithm::SHA1);

    let mut verifier = DetachedVerifierBuilder::from_file(shasums_sig)
        .unwrap()
        .with_policy(&standard_policy, None, node_release_keys.clone())
        .expect("Failed to create Verifier");

    verifier
        .verify_file(&shasums)
        .expect("Failed to verify SHASUMS");

    std::fs::read_to_string(&shasums)
        .expect("Failed to read SHASUMS.txt")
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next(), parts.next()) {
                (Some(checksum), Some(filename), None) => Some((
                    // Some of the checksum filenames contain a leading `./` (e.g.
                    // https://nodejs.org/download/release/v0.11.6/SHASUMS256.txt)
                    filename.trim_start_matches("./").to_string(),
                    checksum.to_string(),
                )),
                _ => None,
            }
        })
        .collect()
}

fn create_http_client() -> ClientWithMiddleware {
    ClientBuilder::new(
        reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(10))
            .build()
            .expect("Should create a reqwest client"),
    )
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(5),
    ))
    .build()
}

async fn download_file(url: impl IntoUrl + std::fmt::Display + Clone) -> PathBuf {
    let client = create_http_client();

    let response = client
        .get(url.clone())
        .send()
        .await
        .unwrap_or_else(|_| panic!("failed to download from {url}"));

    if !response.status().is_success() {
        panic!(
            "Non-successful response code ({}) from {url}",
            response.status()
        );
    }

    let response_data = response
        .bytes()
        .await
        .expect("failed to read response body")
        .to_vec();

    let (mut output_file, output_path) = tempfile::NamedTempFile::new()
        .expect("failed to create temp file")
        .keep()
        .expect("failed to keep temporary file");

    std::io::copy(&mut response_data.as_slice(), &mut output_file)
        .unwrap_or_else(|_| panic!("failed to write response to {}", output_path.display()));

    output_path
}

#[derive(Deserialize, Debug)]
struct NodeRelease {
    pub(crate) version: Version,
    pub(crate) files: Vec<String>,
}

impl NodeRelease {
    fn base_version_url(&self) -> Url {
        BASE_NODEJS_RELEASE_URL
            .join(&format!("v{}/", self.version))
            .unwrap_or_else(|_| panic!("Base version URL should be valid for {}", self.version))
    }

    fn shasums_url(&self) -> Url {
        self.base_version_url()
            .join("SHASUMS256.txt")
            .unwrap_or_else(|_| panic!("SHASUMS256.txt URL should be valid for {}", self.version))
    }

    fn shasums_sig_url(&self) -> Url {
        self.base_version_url()
            .join("SHASUMS256.txt.sig")
            .unwrap_or_else(|_| {
                panic!(
                    "SHASUMS256.txt.sig URL should be valid for {}",
                    self.version
                )
            })
    }

    fn binary_url(&self, supported_platform: &SupportedNodeReleasePlatform) -> Url {
        self.base_version_url()
            .join(&self.binary_file_name(supported_platform))
            .unwrap_or_else(|_| panic!("Binary URL should be valid for {}", self.version))
    }

    fn binary_file_name(
        &self,
        supported_node_release_platform: &SupportedNodeReleasePlatform,
    ) -> String {
        match supported_node_release_platform {
            SupportedNodeReleasePlatform::LinuxX64 => {
                format!("node-v{}-{}.tar.gz", self.version, "linux-x64")
            }
            SupportedNodeReleasePlatform::LinuxArm64 => {
                format!("node-v{}-{}.tar.gz", self.version, "linux-arm64")
            }
        }
    }
}

pub(crate) type NodeArtifact = Artifact<Version, Sha256, Option<()>>;
