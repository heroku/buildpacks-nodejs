use anyhow::anyhow;
use serde::Deserialize;

const NODE_UPSTREAM_LIST_URL: &str = "https://nodejs.org/download/release/index.json";

#[derive(Deserialize)]
pub(crate) struct NodeJSRelease {
    pub(crate) version: String,
}
pub(crate) fn list_releases() -> anyhow::Result<Vec<NodeJSRelease>> {
    ureq::get(NODE_UPSTREAM_LIST_URL)
        .call()
        .map_err(|e| anyhow!("Couldn't fetch nodejs.org release list: {e}"))?
        .into_json::<Vec<NodeJSRelease>>()
        .map_err(|e| anyhow!("Couldn't serialize nodejs.org release list from json: {e}"))
}
