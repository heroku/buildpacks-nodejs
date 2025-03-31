use crate::support::download_file::download_file;
use clap::{ArgMatches, Command, arg};
use regex::Regex;
use sequoia_net::KeyServer;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::parse::stream::{
    DetachedVerifierBuilder, GoodChecksum, MessageLayer, MessageStructure, VerificationHelper,
};
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::{Cert, KeyHandle};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

pub(crate) fn download_verify_node_cmd() -> Command {
    Command::new("download-verify-node")
        .arg(arg!(<version> "version"))
        .arg(arg!(<arch> "arch"))
}

/// See https://github.com/nodejs/node#verifying-binaries
pub(crate) async fn download_verify_node(args: &ArgMatches) {
    let version = args
        .get_one::<String>("version")
        .expect("version argument is required");

    let arch = args
        .get_one::<String>("arch")
        .expect("arch argument is required");

    println!("Importing trusted release keys...");

    let node_release_keys = import_trusted_release_keys().await;

    println!("Downloading Node.js release artifacts...");

    let base_version_url =
        reqwest::Url::parse(&format!("https://nodejs.org/download/release/v{version}/"))
            .expect("base url should be valid");

    // let node_artifact = format!("node-v${version}-${arch}.tar.gz");
    //
    // let node_download = download_file(
    //     base_version_url
    //         .join(&node_artifact)
    //         .expect("download url should be valid"),
    // )
    // .await;

    let shasums = download_file(
        base_version_url
            .join("SHASUMS256.txt")
            .expect("SHASUMS256.txt url should be valid"),
    )
    .await;

    let shasums_sig = download_file(
        base_version_url
            .join("SHASUMS256.txt.sig")
            .expect("SHASUMS256.txt.sig url should be valid"),
    )
    .await;

    println!("Checking Node.js integrity...");
    verify_shasum_signature(node_release_keys, &shasums, &shasums_sig);
    // verify_node_download_checksum(
    //     &node_download,
    //     read_node_artifact_checksum(&shasums, &node_artifact),
    // );
}

async fn import_trusted_release_keys() -> NodeReleaseKeys {
    let keyserver = KeyServer::new("hkps://keys.openpgp.org/").expect("Key server should be valid");

    // See https://github.com/nodejs/node?tab=readme-ov-file#release-keys
    let primary_gpg_keys = "
        gpg --keyserver hkps://keys.openpgp.org --recv-keys C0D6248439F1D5604AAFFB4021D900FFDB233756 # Antoine du Hamel
        gpg --keyserver hkps://keys.openpgp.org --recv-keys DD792F5973C6DE52C432CBDAC77ABFA00DDBF2B7 # Juan José Arboleda
        gpg --keyserver hkps://keys.openpgp.org --recv-keys CC68F5A3106FF448322E48ED27F5E38D5B0A215F # Marco Ippolito
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 # Michaël Zasso
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4 # Rafael Gonzaga
        gpg --keyserver hkps://keys.openpgp.org --recv-keys C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C # Richard Lau
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 108F52B48DB57BB0CC439B2997B01419BD92F80A # Ruy Adorno
        gpg --keyserver hkps://keys.openpgp.org --recv-keys A363A499291CBBC940DD62E41F10027AF002F8B0 # Ulises Gascón
    ".trim().lines().map(|line| {
        let captures = Regex::from_str("gpg --keyserver hkps://keys.openpgp.org --recv-keys (?<key>.*) # (?<owner>.*)")
            .expect("Regex should be valid")
            .captures(line)
            .expect("The line should match the regex pattern");
        (captures["owner"].to_string(), captures["key"].to_string())
    }).collect::<Vec<_>>();

    let mut certs = vec![];

    for (owner, key) in primary_gpg_keys {
        let key_handle = KeyHandle::from_str(&key)
            .unwrap_or_else(|_| panic!("Failed to parse GPG key: {owner} - {key}"));
        let downloaded_certs = keyserver
            .get(key_handle)
            .await
            .unwrap_or_else(|_| panic!("Failed to import GPG key: {owner} - {key}"));
        for downloaded_cert in downloaded_certs {
            match downloaded_cert {
                Ok(cert) => certs.push(cert),
                Err(e) => panic!("Cert error for GPG key: {owner} - {key}\n{e}"),
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

// echo "Verifying Node.js gpg signature..." >&2
// gpg --verify SHASUMS256.txt.sig SHASUMS256.txt

pub(crate) struct NodeReleaseKeys {
    certs: Vec<Cert>,
}

impl NodeReleaseKeys {
    pub(crate) fn new(certs: Vec<Cert>) -> NodeReleaseKeys {
        NodeReleaseKeys { certs }
    }
}

// This was adapted from the example verification process detailed at:
// https://gitlab.com/sequoia-pgp/sequoia/-/blob/main/openpgp/examples/generate-sign-verify.rs
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
                            Ok(GoodChecksum { sig, ka, .. }) => {}
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
        // for (i, layer) in structure.into_iter().enumerate() {
        //     match (i, layer) {
        //         // Consider only level 0 signatures (signatures over the data)
        //         (0, MessageLayer::SignatureGroup { results }) => {
        //             return results
        //                 .into_iter()
        //                 .next()
        //                 .ok_or(anyhow::anyhow!("No signature"))
        //                 .and_then(|verification_result| {
        //                     verification_result
        //                         .map(|_| ())
        //                         .map_err(|e| sequoia_openpgp::Error::from(e).into())
        //                 });
        //         }
        //         _ => Err(anyhow::anyhow!("Unexpected message structure"))?,
        //     }
        // }
        // Err(anyhow::anyhow!("Signature verification failed"))?
    }
}
