use crate::support::download_file::download_file;
use chrono::{DateTime, Utc};
use clap::{ArgMatches, Command, arg};
use regex::Regex;
use sequoia_openpgp::crypto::HashAlgorithm;
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
use std::sync::LazyLock;
use std::time::SystemTime;

pub(crate) fn download_verify_node_cmd() -> Command {
    Command::new("download-verify-node")
        .arg(arg!(<version> "The version of the Node.js binary to download (e.g.; 23.9.0)"))
        .arg(arg!(<arch> "The architecture of the Node.js binary to download (e.g.; arm64, x64, etc.)"))
        .arg(arg!([platform] "The platform of the Node.js binary to download (e.g.; linux, darwin, etc.)").default_value("linux"))
}

/// See https://github.com/nodejs/node#verifying-binaries
pub(crate) async fn download_verify_node(args: &ArgMatches) {
    let version = args
        .get_one::<String>("version")
        .expect("version argument is required");

    let arch = args
        .get_one::<String>("arch")
        .expect("arch argument is required");

    let platform = args
        .get_one::<String>("platform")
        .expect("platform argument is required");

    // Standard Policy rejects SHA1 which has been deprecated since 2013 but many binding signatures
    // still use it for signing. This is a known issue with `sequoia-pgp`:
    // https://gitlab.com/sequoia-pgp/sequoia/-/issues/595
    //
    // > As I understand the attacks on SHA1, SHA1 is susceptible to collisions, but not preimage attacks.
    // > This means that an attacker can create two different documents with the same SHA1 hash, but they
    // > can't create, say, a subkey that has the same SHA1 hash as some other subkey. Thus, it is still
    // > okay to rely on SHA1 to authenticate binding signatures.
    //
    // So support this, we need to configure our policy to accept SHA1, otherwise some Node.js release keys
    // will not be considered valid.
    let mut nodejs_release_policy = StandardPolicy::new();
    nodejs_release_policy.accept_hash(HashAlgorithm::SHA1);

    let base_version_url =
        reqwest::Url::parse(&format!("https://nodejs.org/download/release/v{version}/"))
            .expect("base url should be valid");

    let node_release_artifact_name = format!("node-v{version}-{platform}-{arch}.tar.gz");

    eprintln!("Importing Node.js release keys...");
    let node_release_keys = import_trusted_release_keys(&nodejs_release_policy).await;

    eprintln!("\nDownloading Node.js SHA checksums...");
    let shasums = download_file(
        base_version_url
            .join("SHASUMS256.txt")
            .expect("SHASUMS256.txt url should be valid"),
    )
    .await;

    eprintln!("Downloading Node.js detached signature...");
    let shasums_sig = download_file(
        base_version_url
            .join("SHASUMS256.txt.sig")
            .expect("SHASUMS256.txt.sig url should be valid"),
    )
    .await;

    eprintln!("Verifying SHA checksums signature...");
    verify_shasum_signature(
        node_release_keys,
        &nodejs_release_policy,
        &shasums,
        &shasums_sig,
    );

    eprintln!("\nDownloading Node.js release artifact...");
    let node_release_artifact = download_file(
        base_version_url
            .join(&node_release_artifact_name)
            .expect("download url should be valid"),
    )
    .await;

    eprintln!("Verifying Node.js release artifact checksum...");
    verify_node_download_checksum(
        &node_release_artifact,
        read_node_artifact_checksum(&shasums, &node_release_artifact_name),
    );

    eprintln!("\nOK")
}

async fn import_trusted_release_keys(
    node_release_policy: &StandardPolicy<'_>,
) -> ImportedNodeReleaseKeys {
    let mut imported_release_keys = vec![];

    for node_release_key in NODE_RELEASE_KEYS.trim().lines().map(NodeReleaseKey::from) {
        let cert =
            Cert::from_file(download_file(&node_release_key.url).await).unwrap_or_else(|e| {
                panic!(
                    "Could not load release key from {} ({})\nError: {e}\n{}",
                    node_release_key.url,
                    node_release_key.owner,
                    e.chain()
                        .skip(1)
                        .map(|cause| format!("because: {cause}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                );
            });

        let imported_release_key = ImportedNodeReleaseKey {
            node_release_key,
            cert,
        };

        if let Err(e) = imported_release_key
            .cert
            .with_policy(node_release_policy, None)
        {
            panic!(
                "The following release key was not valid according to the given policy:\n\n{imported_release_key}\n\nError: {e}\n{}",
                e.chain()
                    .skip(1)
                    .map(|cause| format!("because: {cause}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }

        eprintln!(
            "- Imported {} <{}> (created {})",
            imported_release_key.node_release_key.fingerprint,
            imported_release_key.node_release_key.owner,
            display_system_time(
                imported_release_key
                    .cert
                    .primary_key()
                    .component()
                    .creation_time()
            )
        );

        imported_release_keys.push(imported_release_key);
    }

    ImportedNodeReleaseKeys {
        inner: imported_release_keys,
    }
}

fn verify_shasum_signature(
    node_release_keys: ImportedNodeReleaseKeys,
    node_release_policy: &StandardPolicy<'_>,
    shasums_path: &Path,
    shasums_sig_path: &Path,
) {
    let mut verifier = DetachedVerifierBuilder::from_file(shasums_sig_path)
        .unwrap()
        .with_policy(node_release_policy, None, node_release_keys)
        .expect("Failed to create Verifier");

    verifier.verify_file(shasums_path).unwrap_or_else(|_| {
        panic!(
            "Failed to verify {}",
            shasums_path
                .file_name()
                .expect("Missing file name")
                .to_string_lossy()
        )
    });
}

fn read_node_artifact_checksum(shasums_path: &Path, node_artifact: &str) -> String {
    fs::read_to_string(shasums_path)
        .expect("Failed to read SHASUMS256.txt")
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

static NODE_RELEASE_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::from_str(
        "gpg --keyserver (?<keyserver>.*) --recv-keys (?<fingerprint>.*) # (?<owner>.*)",
    )
    .expect("Node.js release key regex should be valid")
});

struct NodeReleaseKey {
    fingerprint: String,
    url: String,
    owner: String,
}

impl From<&str> for NodeReleaseKey {
    fn from(value: &str) -> Self {
        let captures = NODE_RELEASE_KEY_REGEX
            .captures(value)
            .expect("The line should match the regex pattern");

        let fingerprint = captures["fingerprint"].to_string();
        let owner = captures["owner"].to_string();
        let keyserver = captures["keyserver"].to_string();

        // We need to confirm that there's not another keyserver being used because this script
        // doesn't support that.
        if keyserver != "hkps://keys.openpgp.org" {
            panic!("Unsupported keyserver: {}", &keyserver);
        }

        // Instead of pulling in `sequoia-net` to fetch keys, it's easier to just use the REST API
        // url from https://keys.openpgp.org/ for fetching the key by fingerprint.
        let url = format!("https://keys.openpgp.org/vks/v1/by-fingerprint/{fingerprint}");
        NodeReleaseKey {
            fingerprint,
            owner,
            url,
        }
    }
}

struct ImportedNodeReleaseKey {
    node_release_key: NodeReleaseKey,
    cert: Cert,
}

impl Display for ImportedNodeReleaseKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cert = &self.cert;

        writeln!(f, "-----")?;
        writeln!(f, "Primary key:")?;
        writeln!(f, "  Fingerprint: {}", cert.fingerprint())?;
        writeln!(
            f,
            "  Creation time: {}",
            display_system_time(cert.primary_key().component().creation_time())
        )?;

        writeln!(f, "User IDs:")?;
        for uid in cert.userids() {
            writeln!(f, "  {}", uid.userid())?;
        }

        // List all subkeys
        writeln!(f, "Subkeys:")?;
        for subkey in cert.keys().subkeys() {
            writeln!(
                f,
                "  Subkey fingerprint: {}",
                subkey.component().fingerprint()
            )?;
            writeln!(
                f,
                "  Creation time: {}",
                display_system_time(subkey.component().creation_time())
            )?;

            for signature in subkey.signatures() {
                writeln!(f, "    Signature:")?;
                writeln!(f, "      Type: {:?}", signature.typ())?;
                if let Some(creation_time) = signature.signature_creation_time() {
                    writeln!(
                        f,
                        "      Creation time: {}",
                        display_system_time(creation_time)
                    )?;
                } else {
                    writeln!(f, "      Creation time: None")?;
                }
                if let Some(exp) = signature.signature_expiration_time() {
                    writeln!(f, "      Expiration time: {}", display_system_time(exp))?;
                } else {
                    writeln!(f, "      Expiration time: None")?;
                }
                writeln!(f, "      Algorithm: {}", signature.hash_algo())?;
                if let Some(flags) = signature.key_flags() {
                    writeln!(f, "      Key flags:")?;
                    if flags.for_certification() {
                        writeln!(f, "        - Can certify other keys")?;
                    }
                    if flags.for_signing() {
                        writeln!(f, "        - Can sign")?;
                    }
                    if flags.for_transport_encryption() {
                        writeln!(f, "        - Can encrypt communications")?;
                    }
                    if flags.for_storage_encryption() {
                        writeln!(f, "        - Can encrypt storage")?;
                    }
                    if flags.for_authentication() {
                        writeln!(f, "        - Can authenticate")?;
                    }
                }
            }
        }

        writeln!(f, "-----")
    }
}

struct ImportedNodeReleaseKeys {
    inner: Vec<ImportedNodeReleaseKey>,
}

// This was adapted from the example verification process detailed at:
// https://gitlab.com/sequoia-pgp/sequoia/-/blob/main/openpgp/examples/generate-sign-verify.rs
impl VerificationHelper for ImportedNodeReleaseKeys {
    fn get_certs(&mut self, _: &[KeyHandle]) -> sequoia_openpgp::Result<Vec<Cert>> {
        Ok(self
            .inner
            .iter()
            .map(|imported_release_key| imported_release_key.cert.clone())
            .collect())
    }

    fn check(&mut self, structure: MessageStructure) -> sequoia_openpgp::Result<()> {
        let mut found_valid_signature = false;
        let key_handles = self
            .inner
            .iter()
            .flat_map(|imported_release_key| {
                imported_release_key
                    .cert
                    .keys()
                    .map(|ka| ka.key().key_handle())
            })
            .collect::<Vec<_>>();

        for layer in structure.into_iter() {
            match layer {
                MessageLayer::SignatureGroup { results } => {
                    for result in results {
                        match result {
                            Ok(GoodChecksum { ka, .. }) => {
                                // Check if the signature was made by one of our trusted keys
                                if key_handles
                                    .iter()
                                    .any(|key_handle| key_handle == &ka.key().key_handle())
                                {
                                    found_valid_signature = true;
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("Signature verification error: {}", e);
                            }
                        }
                    }
                    if found_valid_signature {
                        break;
                    }
                }
                MessageLayer::Compression { .. } => (),
                _ => unreachable!(),
            }
        }

        if !found_valid_signature {
            return Err(sequoia_openpgp::Error::InvalidArgument(
                "No valid signature found from trusted Node.js release keys".into(),
            )
            .into());
        }

        Ok(())
    }
}

fn display_system_time(system_time: SystemTime) -> String {
    let date_time: DateTime<Utc> = DateTime::from(system_time);
    date_time.to_rfc3339()
}
