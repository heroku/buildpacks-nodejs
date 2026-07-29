use regex::Regex;
use sequoia_net::KeyServer;
use sequoia_openpgp::parse::stream::{
    MessageLayer, MessageStructure, VerificationError, VerificationHelper,
};
use sequoia_openpgp::{Cert, KeyHandle};
use std::str::FromStr;

pub(super) async fn import() -> NodeReleaseKeys {
    // See https://github.com/nodejs/node?tab=readme-ov-file#release-keys
    let documented_node_release_keys = "
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 5BE8A3F6C8A5C01D106C0AD820B1A390B168D356 # Antoine du Hamel
        gpg --keyserver hkps://keys.openpgp.org --recv-keys DD792F5973C6DE52C432CBDAC77ABFA00DDBF2B7 # Juan José Arboleda
        gpg --keyserver hkps://keys.openpgp.org --recv-keys CC68F5A3106FF448322E48ED27F5E38D5B0A215F # Marco Ippolito
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 8FCCA13FEF1D0C2E91008E09770F7A9A5AE15600 # Michaël Zasso
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 890C08DB8579162FEE0DF9DB8BEAB4DFCF555EF4 # Rafael Gonzaga
        gpg --keyserver hkps://keys.openpgp.org --recv-keys C82FA3AE1CBEDC6BE46B9360C43CEC45C17AB93C # Richard Lau
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 108F52B48DB57BB0CC439B2997B01419BD92F80A # Ruy Adorno
        gpg --keyserver hkps://keys.openpgp.org --recv-keys 655F3B5C1FB3FA8D1A0CA6BDE4A7D232B936D2FD # Stewart X Addison
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
                            if is_acceptable_expired_key_signature(&e) {
                                eprintln!(
                                    "Warning: accepting a signature from an expired Node.js release key (the signature is still cryptographically valid): {e}"
                                );
                            } else {
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

/// `gpg --verify` treats a signature from an expired key as valid (with a warning) as long as the
/// signature itself is cryptographically sound. This has happened with a real Node.js release: the
/// SHASUMS for v26.5.1 were signed on 2026-07-29 by a key that expired 2026-07-08. Sequoia's
/// verifier is stricter — it short-circuits on the expired key and never reaches the cryptographic
/// check, surfacing it as `VerificationError::BadKey`. To match `gpg`'s behavior without weakening
/// our integrity guarantee, we accept a `BadKey` failure *only* when the key is expired (not revoked
/// or otherwise unusable) *and* the signature is still cryptographically valid against the signed
/// data.
fn is_acceptable_expired_key_signature(error: &VerificationError<'_>) -> bool {
    let VerificationError::BadKey { sig, ka, error } = error else {
        return false;
    };
    // `BadKey` also covers revocation and non-signing-capable keys, which must remain hard failures.
    // Only an expiration error (wrapped by Sequoia in a "not live" context) is acceptable.
    let is_expired = error.chain().any(|cause| {
        matches!(
            cause.downcast_ref::<sequoia_openpgp::Error>(),
            Some(sequoia_openpgp::Error::Expired(_))
        )
    });
    // Re-run the cryptographic verification that Sequoia skipped once it decided the key was expired.
    // The signature's digest was computed during streaming, so this checks the signature against the
    // actual signed content.
    is_expired && sig.verify_document(ka.key()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sequoia_openpgp::cert::CertBuilder;
    use sequoia_openpgp::crypto::HashAlgorithm;
    use sequoia_openpgp::parse::Parse;
    use sequoia_openpgp::parse::stream::DetachedVerifierBuilder;
    use sequoia_openpgp::policy::StandardPolicy;
    use sequoia_openpgp::serialize::stream::{Message, Signer};
    use sequoia_openpgp::types::KeyFlags;
    use std::io::Write;
    use std::time::{Duration, SystemTime};

    // Builds a signing-capable certificate that expired an hour ago, then produces a detached
    // signature over `payload`. This mirrors the real Node.js scenario where a release signer's key
    // has lapsed but they keep signing new releases with it.
    fn expired_key_detached_signature(payload: &[u8]) -> (Cert, Vec<u8>) {
        let created = SystemTime::now() - Duration::from_secs(60 * 60 * 24);
        let cert = CertBuilder::new()
            .set_creation_time(created)
            .set_validity_period(Duration::from_secs(60 * 60 * 23))
            .set_primary_key_flags(KeyFlags::empty().set_signing())
            .generate()
            .expect("should generate a certificate")
            .0;

        let keypair = cert
            .primary_key()
            .key()
            .clone()
            .parts_into_secret()
            .expect("generated cert should have a secret key")
            .into_keypair()
            .expect("should build a keypair");

        let mut signature = vec![];
        let message = Message::new(&mut signature);
        let mut signer = Signer::new(message, keypair)
            .expect("should build a signer")
            .detached()
            .build()
            .expect("should build a detached signer");
        signer.write_all(payload).expect("should sign payload");
        signer.finalize().expect("should finalize signature");

        (cert, signature)
    }

    fn verify(cert: &Cert, signature: &[u8], payload: &[u8]) -> sequoia_openpgp::Result<()> {
        let mut policy = StandardPolicy::new();
        policy.accept_hash(HashAlgorithm::SHA1);
        let keys = NodeReleaseKeys {
            certs: vec![cert.clone()],
        };
        DetachedVerifierBuilder::from_bytes(signature)?
            .with_policy(&policy, None, keys)?
            .verify_bytes(payload)
    }

    // A cryptographically valid signature from an expired key is accepted, matching `gpg --verify`,
    // which treats key expiration as a warning rather than a failure.
    #[test]
    fn accepts_valid_signature_from_expired_key() {
        let payload = b"deadbeef  node-v99.0.0-linux-x64.tar.gz\n";
        let (cert, signature) = expired_key_detached_signature(payload);
        verify(&cert, &signature, payload)
            .expect("valid signature from an expired key is accepted");
    }

    // A tampered payload must still be rejected even when the signing key is expired — the expired
    // key exception must not become a way to accept unverified content.
    #[test]
    #[should_panic(expected = "Signature error")]
    fn rejects_tampered_payload_from_expired_key() {
        let payload = b"deadbeef  node-v99.0.0-linux-x64.tar.gz\n";
        let (cert, signature) = expired_key_detached_signature(payload);
        let tampered = b"ba5eba11  node-v99.0.0-linux-x64.tar.gz\n";
        let _ = verify(&cert, &signature, tampered);
    }
}
