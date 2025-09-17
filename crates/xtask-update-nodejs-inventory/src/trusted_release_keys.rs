use regex::Regex;
use sequoia_net::KeyServer;
use sequoia_openpgp::parse::stream::{MessageLayer, MessageStructure, VerificationHelper};
use sequoia_openpgp::{Cert, KeyHandle};
use std::str::FromStr;

pub(super) async fn import() -> NodeReleaseKeys {
    // See https://github.com/nodejs/node?tab=readme-ov-file#release-keys
    let documented_node_release_keys = "
        gpg --keyserver hkps://keys.openpgp.org --recv-keys C0D6248439F1D5604AAFFB4021D900FFDB233756 # Antoine du Hamel
        gpg --keyserver hkps://keys.openpgp.org --recv-keys DD792F5973C6DE52C432CBDAC77ABFA00DDBF2B7 # Juan José Arboleda
        gpg --keyserver hkps://keys.openpgp.org --recv-keys CC68F5A3106FF448322E48ED27F5E38D5B0A215F # Marco Ippolito
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 # Michaël Zasso
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4 # Rafael Gonzaga
        gpg --keyserver hkps://keys.openpgp.org --recv-keys C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C # Richard Lau
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 108F52B48DB57BB0CC439B2997B01419BD92F80A # Ruy Adorno
        gpg --keyserver hkps://keys.openpgp.org --recv-keys A363A499291CBBC940DD62E41F10027AF002F8B0 # Ulises Gascón
    ".trim().lines().map(|line| {
        let captures = Regex::from_str("gpg --keyserver (?<key_server>.*) --recv-keys (?<key>.*) # (?<owner>.*)")
            .expect("Regex should be valid")
            .captures(line)
            .expect("The line should match the regex pattern");
        DocumentedNodeReleaseKey {
            key_server: captures["key_server"].to_string(),
            key: captures["key"].to_string(),
            owner: captures["owner"].to_string()
        }
    }).collect::<Vec<_>>();

    let mut certs = vec![];

    for DocumentedNodeReleaseKey {
        key_server,
        key,
        owner,
    } in documented_node_release_keys
    {
        let keyserver = KeyServer::new(&key_server).expect("Key server should be valid");

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

pub(super) struct DocumentedNodeReleaseKey {
    key_server: String,
    key: String,
    owner: String,
}

#[derive(Clone)]
pub(super) struct NodeReleaseKeys {
    certs: Vec<Cert>,
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
                        if let Err(e) = result {
                            panic!("Signature error: {e}")
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
