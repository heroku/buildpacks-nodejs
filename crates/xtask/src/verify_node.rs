use crate::support::args::version_arg;
use crate::support::download_file::download_file;
use clap::{Parser, ValueEnum, arg};
use node_semver::Version;
use regex::Regex;
use sequoia_net::KeyServer;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::parse::stream::{
    DetachedVerifierBuilder, GoodChecksum, MessageLayer, MessageStructure, VerificationHelper,
};
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::{Cert, KeyHandle};
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::str::FromStr;

// See https://github.com/nodejs/node?tab=readme-ov-file#release-keys
const NODE_RELEASE_KEYS: &str = "
    gpg --keyserver hkps://keys.openpgp.org --recv-keys C0D6248439F1D5604AAFFB4021D900FFDB233756 # Antoine du Hamel
    gpg --keyserver hkps://keys.openpgp.org --recv-keys DD792F5973C6DE52C432CBDAC77ABFA00DDBF2B7 # Juan José Arboleda
    gpg --keyserver hkps://keys.openpgp.org --recv-keys CC68F5A3106FF448322E48ED27F5E38D5B0A215F # Marco Ippolito
    gpg --keyserver hkps://keys.openpgp.org --recv-keys 8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 # Michaël Zasso
    gpg --keyserver hkps://keys.openpgp.org --recv-keys 890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4 # Rafael Gonzaga
    gpg --keyserver hkps://keys.openpgp.org --recv-keys C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C # Richard Lau
    gpg --keyserver hkps://keys.openpgp.org --recv-keys 108F52B48DB57BB0CC439B2997B01419BD92F80A # Ruy Adorno
    gpg --keyserver hkps://keys.openpgp.org --recv-keys A363A499291CBBC940DD62E41F10027AF002F8B0 # Ulises Gascón
";

#[derive(Parser, Debug)]
pub(super) struct VerifyNodeArgs {
    #[arg(long, value_parser = version_arg)]
    version: Version,

    #[arg(long, value_enum)]
    arch: SupportedNodeJsArchitecture,
}

/// See https://github.com/nodejs/node#verifying-binaries
pub(crate) async fn verify_node(args: &VerifyNodeArgs) {
    let VerifyNodeArgs { version, arch } = args;
    let node_artifact = format!("node-v{version}-linux-{arch}.tar.gz");
    let base_url = reqwest::Url::parse(&format!("https://nodejs.org/download/release/v{version}/"))
        .expect("base url should be valid");

    print!("Importing trusted release keys ... ");
    let node_release_keys = import_trusted_release_keys().await;
    println!("✅ ");

    print!("Downloading release checksums ... ");
    let shasums_url = base_url
        .join("SHASUMS256.txt")
        .expect("SHASUMS256.txt url should be valid");
    let shasums = download_file(shasums_url).await;
    println!("✅ ");

    print!("Download detached GPG signature ... ");
    let shasums_sig_url = base_url
        .join("SHASUMS256.txt.sig")
        .expect("SHASUMS256.txt.sig url should be valid");
    let shasums_sig = download_file(shasums_sig_url).await;
    println!("✅ ");

    print!("Verifying release checksums ... ");
    verify_shasum_signature(node_release_keys, &shasums, &shasums_sig);
    println!("✅ ");

    print!("Downloading Node.js release ... ");
    let node_download_url = base_url
        .join(&node_artifact)
        .expect("download url should be valid");
    let node_download = download_file(node_download_url).await;
    println!("✅ ");

    print!("Checking Node.js release checksum ... ");
    verify_node_download_checksum(
        &node_download,
        read_node_artifact_checksum(&shasums, &node_artifact),
    );
    println!("✅ ");
}

async fn import_trusted_release_keys() -> NodeReleaseKeys {
    let primary_gpg_keys = NODE_RELEASE_KEYS
        .trim()
        .lines()
        .map(|line| {
            let captures = Regex::from_str(
                "gpg --keyserver (?<keyserver>.*) --recv-keys (?<key>.*) # (?<owner>.*)",
            )
            .expect("Regex should be valid")
            .captures(line)
            .expect("The line should match the regex pattern");
            let keyserver = captures["keyserver"].to_string();
            let key = captures["key"].to_string();
            let owner = captures["owner"].to_string();
            (
                KeyServer::new(&keyserver).unwrap_or_else(|_| {
                    panic!("Key server should be valid: {keyserver} # {owner}")
                }),
                KeyHandle::from_str(&key)
                    .unwrap_or_else(|_| panic!("Failed to parse GPG key: {key} #{owner}")),
                owner,
            )
        })
        .collect::<Vec<_>>();

    let mut certs = vec![];

    for (keyserver, key_handle, owner) in primary_gpg_keys {
        let downloaded_certs = keyserver
            .get(key_handle.clone())
            .await
            .unwrap_or_else(|e| panic!("Failed to import GPG key: {owner} - {key_handle}\n{e}"));
        for downloaded_cert in downloaded_certs {
            match downloaded_cert {
                Ok(cert) => certs.push(cert),
                Err(e) => panic!("Cert error for GPG key: {owner} - {key_handle}\n{e}"),
            }
        }
    }

    NodeReleaseKeys { certs }
}

fn read_node_artifact_checksum(shasums_path: &Path, node_artifact: &str) -> String {
    fs::read_to_string(shasums_path)
        .expect("Failed to read SHASUMS256.txt.txt")
        .lines()
        .find_map(|line| {
            let mut shasum_parts = line.split_whitespace();

            let checksum = shasum_parts
                .next()
                .expect("SHASUMS256.txt should contain a checksum in the first part of a line");

            let target_artifact = shasum_parts
                .next()
                .expect("SHASUMS256.txt should contain a node artifact filename in the second part of a line");

            if target_artifact == node_artifact {
                Some(checksum)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("No shasum found for artifact: {node_artifact}")).to_string()
}

fn verify_node_download_checksum(node_download_path: &Path, expected_checksum: String) {
    let actual_checksum = sha256::try_digest(node_download_path).unwrap_or_else(|_| {
        panic!(
            "Unable to create digest for {}",
            node_download_path.display()
        )
    });
    if actual_checksum != expected_checksum {
        panic!(
            "Node download digest mismatch.\nExpected: {expected_checksum}\nGot:      {actual_checksum}"
        )
    }
}

/// Does the same verification as `gpg --verify SHASUMS256.txt.sig SHASUMS256.txt`
fn verify_shasum_signature(
    node_release_keys: NodeReleaseKeys,
    shasums_path: &Path,
    shasums_sig_path: &Path,
) {
    let standard_policy = StandardPolicy::new();

    let mut verifier = DetachedVerifierBuilder::from_file(shasums_sig_path)
        .unwrap()
        .with_policy(&standard_policy, None, node_release_keys)
        .expect("Failed to create Verifier");

    verifier
        .verify_file(shasums_path)
        .expect("Failed to verify SHASUMS");
}

pub(crate) struct NodeReleaseKeys {
    certs: Vec<Cert>,
}

impl VerificationHelper for NodeReleaseKeys {
    fn get_certs(&mut self, _: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
        Ok(self.certs.clone())
    }

    fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
        for layer in structure.into_iter() {
            match layer {
                MessageLayer::SignatureGroup { results } => {
                    for result in results {
                        match result {
                            Ok(GoodChecksum { .. }) => {}
                            Err(e) => {
                                panic!("Signature error: {e}")
                            }
                        }
                    }
                }
                MessageLayer::Compression { .. } => (),
                _ => unreachable!(),
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum SupportedNodeJsArchitecture {
    Arm64,
    X64,
}

impl Display for SupportedNodeJsArchitecture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupportedNodeJsArchitecture::Arm64 => write!(f, "arm64"),
            SupportedNodeJsArchitecture::X64 => write!(f, "x64"),
        }
    }
}
