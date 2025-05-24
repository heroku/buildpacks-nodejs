use crate::vrs::Version;
use anyhow::anyhow;
use serde::Deserialize;
use std::collections::HashMap;

const NPMJS_ORG_HOST: &str = "https://registry.npmjs.org";

#[derive(Deserialize)]
struct NpmPackage {
    versions: HashMap<String, NpmRelease>,
}

#[derive(Deserialize)]
pub(crate) struct NpmRelease {
    pub(crate) version: Version,
}

pub(crate) fn list_releases(package: &str) -> anyhow::Result<Vec<NpmRelease>> {
    ureq::get(&format!("{NPMJS_ORG_HOST}/{package}"))
        .call()
        .map_err(|e| anyhow!("Couldn't fetch npmjs registry release list from for {package}: {e}"))?
        .body_mut()
        .read_json::<NpmPackage>()
        .map_err(|e| anyhow!("Couldn't serialize npmjs registry release list for {package}: {e}"))
        .map(|rel| rel.versions.into_values().collect())
}
