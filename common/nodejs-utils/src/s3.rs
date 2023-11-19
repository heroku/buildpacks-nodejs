use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

/// Content Node in the XML document returned by Amazon S3 for a public bucket.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub(crate) struct Content {
    // Examples of keys:
    // * npm/release/npm-v9.7.2.tar.gz
    // * yarn/release/yarn-v0.16.0.tar.gz
    // * node/release/darwin-x64/node-v0.10.0-darwin-x64.tar.gz
    pub(crate) key: String,
    pub(crate) last_modified: DateTime<Utc>,
    #[serde(rename = "ETag")]
    pub(crate) etag: String,
    pub(crate) size: usize,
    pub(crate) storage_class: String,
}

/// Representation of the XML document returned by Amazon S3 for a public bucket.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
struct ListBucketResult {
    name: String,
    pub(crate) prefix: String,
    max_keys: usize,
    is_truncated: bool,
    continuation_token: Option<String>,
    next_continuation_token: Option<String>,
    contents: Option<Vec<Content>>,
}

/// Represents contents of a bucket's prefix
#[derive(Debug)]
pub(crate) struct BucketContent {
    pub(crate) name: String,
    pub(crate) region: String,
    pub(crate) prefix: String,
    pub(crate) contents: Vec<Content>,
}

impl Default for BucketContent {
    fn default() -> Self {
        Self {
            name: "heroku-nodebin".to_string(),
            region: "us-east-1".to_string(),
            prefix: String::new(),
            contents: vec![],
        }
    }
}

/// Fetch all s3 objects for a given folder.
///
/// # Errors
///
/// * Failed http requests
/// * Parsing errors for an invalid S3 URL
/// * XML Parsing errors for an invalid XML document
pub(crate) fn list_objects<B: AsRef<str>, R: AsRef<str>, P: AsRef<str>>(
    b: B,
    r: R,
    p: P,
) -> anyhow::Result<BucketContent> {
    let bucket = b.as_ref();
    let region = r.as_ref();
    let prefix = p.as_ref();

    let mut bucket_content = BucketContent {
        name: bucket.to_string(),
        region: region.to_string(),
        prefix: prefix.to_string(),
        contents: vec![],
    };
    let mut continuation_token = String::new();
    loop {
        let mut params = vec![("prefix", prefix), ("list-type", "2")];
        if !continuation_token.is_empty() {
            params.push(("continuation-token", continuation_token.as_str()));
        }

        let url = Url::parse_with_params(
            &format!("https://{bucket}.s3.{region}.amazonaws.com/"),
            params,
        )?;
        let res = ureq::get(url.as_ref()).call()?.into_string()?;
        let page: ListBucketResult = serde_xml_rs::from_str(&res)?;
        if let Some(mut contents) = page.contents {
            bucket_content.contents.append(&mut contents);
        }

        match page.next_continuation_token {
            None => break,
            Some(token) => continuation_token = token,
        }
    }
    Ok(bucket_content)
}
