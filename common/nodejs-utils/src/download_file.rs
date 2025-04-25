use reqwest::IntoUrl;
use reqwest_middleware::ClientBuilder;
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(10);

const DEFAULT_RETRIES: u32 = 5;

#[bon::builder]
pub fn download_file_sync(
    from_url: impl IntoUrl + std::fmt::Display + Clone,
    to_file: impl AsRef<Path>,
    #[builder(default = DEFAULT_CONNECT_TIMEOUT)] connect_timeout: Duration,
    #[builder(default = DEFAULT_READ_TIMEOUT)] read_timeout: Duration,
    #[builder(default = DEFAULT_RETRIES)] max_retries: u32,
) -> Result<(), DownloadError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Should be able to construct the Async Runtime");

    runtime.block_on(async {
        download_file()
            .from_url(from_url)
            .to_file(to_file)
            .connect_timeout(connect_timeout)
            .read_timeout(read_timeout)
            .max_retries(max_retries)
            .call()
            .await
    })
}

#[bon::builder]
pub async fn download_file(
    from_url: impl IntoUrl + std::fmt::Display + Clone,
    to_file: impl AsRef<Path>,
    #[builder(default = DEFAULT_CONNECT_TIMEOUT)] connect_timeout: Duration,
    #[builder(default = DEFAULT_READ_TIMEOUT)] read_timeout: Duration,
    #[builder(default = DEFAULT_RETRIES)] max_retries: u32,
) -> Result<(), DownloadError> {
    let to_file = to_file.as_ref();

    let client = ClientBuilder::new(
        reqwest::ClientBuilder::new()
            .use_rustls_tls()
            .connect_timeout(connect_timeout)
            .read_timeout(read_timeout)
            .build()
            .expect("Should be able to construct the HTTP client"),
    )
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(max_retries),
    ))
    .build();

    let response = client
        .get(from_url.clone())
        .send()
        .await
        .and_then(|res| {
            res.error_for_status()
                .map_err(reqwest_middleware::Error::Reqwest)
        })
        .map_err(|e| DownloadError::Request(from_url.to_string(), e))?;

    let response_data = response
        .bytes()
        .await
        .map_err(|e| {
            DownloadError::ReadResponse(from_url.to_string(), reqwest_middleware::Error::Reqwest(e))
        })?
        .to_vec();

    let mut output_file = File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to_file)
        .map_err(|e| DownloadError::OpenFile(to_file.to_path_buf(), from_url.to_string(), e))?;

    io::copy(&mut response_data.as_slice(), &mut output_file)
        .map_err(|e| DownloadError::WriteFile(to_file.to_path_buf(), from_url.to_string(), e))?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("Request to `{0}` failed\nError: {1}")]
    Request(String, reqwest_middleware::Error),
    #[error("Reading response from request to `{0}` failed\nError: {1}")]
    ReadResponse(String, reqwest_middleware::Error),
    #[error("Could not open file at `{0}` for download of `{1}`\nError: {2}")]
    OpenFile(PathBuf, String, io::Error),
    #[error("Could not write to file at `{0}` for download of `{1}`\nError: {2}")]
    WriteFile(PathBuf, String, io::Error),
}
