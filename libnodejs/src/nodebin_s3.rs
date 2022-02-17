//! A module wrapping up S3 functionality

use anyhow::{anyhow, Error};
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::Url;
use semver::Version;
use serde::Deserialize;
use std::convert::TryFrom;

use crate::resolve_version::{Release, SemVersion, Software, BUCKET};

/// Content Node in the XML document returned by Amazon S3 for a public bucket.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct Content {
    // Examples of keys:
    // * yarn/release/yarn-v0.16.0.tar.gz
    // * node/release/darwin-x64/node-v0.10.0-darwin-x64.tar.gz
    key: String,
    last_modified: DateTime<Utc>,
    #[serde(rename = "ETag")]
    etag: String,
    size: usize,
    storage_class: String,
}

/// Representation of the XML document returned by Amazon S3 for a public bucket.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct ListBucketResult {
    name: String,
    prefix: String,
    max_keys: usize,
    is_truncated: bool,
    continuation_token: Option<String>,
    next_continuation_token: Option<String>,
    contents: Vec<Content>,
}

/// Represents contents of a bucket's prefix
#[derive(Debug)]
pub struct BucketContent {
    prefix: String,
    contents: Vec<Content>,
}

impl TryFrom<BucketContent> for Software {
    type Error = anyhow::Error;

    /// # Failures
    /// These are the possible errors that can occur when calling this function:
    ///
    /// * Regex missing matching captures against `Content#key`
    /// * `Version::parse` fails to parse the version found in the `Content#key`
    fn try_from(result: BucketContent) -> Result<Self, Self::Error> {
        let software = &result.prefix;
        let version_regex = Regex::new(&format!(
            r"{}/(?P<channel>\w+)/(?P<arch>[\w-]+)?/?{}-v(?P<version>\d+\.\d+\.\d+)([\w-]+)?\.tar\.gz",
            software, software
        ))?;

        let releases: Result<Vec<Release>, Error> = result
            .contents
            .iter()
            .map(|content| {
                let capture = version_regex.captures(&content.key).ok_or_else(|| {
                    anyhow!("No valid version found in content: {}", &content.key)
                })?;
                let channel = capture.name("channel").ok_or_else(|| {
                    anyhow!("Could not find channel in content: {}", &content.key)
                })?;
                let version_number = capture.name("version").ok_or_else(|| {
                    anyhow!("Could not find version in content: {}", &content.key)
                })?;
                let arch = capture.name("arch");

                Ok(Release {
                    arch: arch.map(|a| a.as_str().to_string()),
                    version: SemVersion::new(Version::parse(version_number.as_str())?),
                    channel: channel.as_str().to_string(),
                    // Amazon S3 returns a quoted string for ETags
                    etag: content.etag.replace("\"", ""),
                    url: format!("https://s3.amazonaws.com/{}/{}", BUCKET, &content.key),
                })
            })
            .collect();

        Ok(Self {
            name: software.to_string(),
            releases: releases?,
        })
    }
}

/// Fetch all s3 buckets for a given folder.
///
/// # Failures
/// These are the possible errors that can occur when calling this function:
///
/// * Parsing errors for an invalid S3 URL.
/// * XML Parsing errors for an invalid XML document.
pub async fn list_s3_objects<B: AsRef<str>, R: AsRef<str>, P: AsRef<str>>(
    b: B,
    r: R,
    p: P,
) -> Result<BucketContent, Error> {
    let bucket = b.as_ref();
    let region = r.as_ref();
    let prefix = p.as_ref();

    let mut bucket_content = BucketContent {
        prefix: prefix.to_string(),
        contents: vec![],
    };
    let mut continuation_token = "".to_string();
    loop {
        let mut params = vec![("prefix", prefix), ("list-type", "2")];
        if continuation_token != "" {
            params.push(("continuation-token", continuation_token.as_str()));
        }
        let url = Url::parse_with_params(
            &format!("https://{}.s3.{}.amazonaws.com", bucket, region),
            &params,
        )?;
        let mut page: ListBucketResult =
            serde_xml_rs::from_str(&reqwest::get(url).await?.text().await?)?;
        bucket_content.contents.append(&mut page.contents);

        match page.next_continuation_token {
            None => break,
            Some(token) => continuation_token = token,
        }
    }
    Ok(bucket_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn it_converts_s3_result_to_software() {
        let etag = "739c200ca266266ff150ad4d89b83205";
        let content = Content {
            key: "node/release/darwin-x64/node-v0.10.0-darwin-x64.tar.gz".to_string(),
            last_modified: Utc::now(),
            etag: format!("\"{}\"", etag),
            size: 4_065_868,
            storage_class: "STANDARD".to_string(),
        };
        let bucket_content = BucketContent {
            prefix: "node".to_string(),
            contents: vec![content],
        };

        let result = Software::try_from(bucket_content);
        assert!(result.is_ok());
        if let Ok(software) = result {
            assert_eq!(etag, software.releases[0].etag);
        }
    }

    #[test]
    fn it_converts_s3_result_to_software_arch_optional() {
        let content = Content {
            key: "yarn/release/yarn-v0.16.0.tar.gz".to_string(),
            last_modified: Utc::now(),
            etag: "\"e4cc76bea92fabb664edadc4db14a8f2\"".to_string(),
            size: 7_234_362,
            storage_class: "STANDARD".to_string(),
        };
        let bucket_content = BucketContent {
            prefix: "yarn".to_string(),
            contents: vec![content],
        };

        let result = Software::try_from(bucket_content);
        assert!(result.is_ok());
    }

    #[test]
    fn it_fails_to_convert_s3_result_to_software() {
        let content = Content {
            key: "garbage".to_string(),
            last_modified: Utc::now(),
            etag: "\"e4cc76bea92fabb664edadc4db14a8f2\"".to_string(),
            size: 7_234_362,
            storage_class: "STANDARD".to_string(),
        };
        let bucket_content = BucketContent {
            prefix: "yarn".to_string(),
            contents: vec![content],
        };

        let result = Software::try_from(bucket_content);
        assert!(result.is_err());
    }
}
