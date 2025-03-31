use crate::support::create_http_client::create_http_client;
use reqwest::IntoUrl;
use std::io;
use std::io::BufReader;
use std::path::PathBuf;

pub(crate) async fn download_file(url: impl IntoUrl + std::fmt::Display + Clone) -> PathBuf {
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

    io::copy(&mut response_data.as_slice(), &mut output_file)
        .unwrap_or_else(|_| panic!("failed to write response to {}", output_path.display()));

    output_path
}
