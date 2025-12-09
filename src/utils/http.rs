use bullet_stream::global::print;
use bullet_stream::{GlobalTimer, style};
use flate2::read::MultiGzDecoder;
use http::{HeaderMap, StatusCode};
use reqwest::blocking::Response;
use retry::delay::Fixed;
use retry::{OperationResult, retry_with_index};
use sha2::{Digest, Sha256};
use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::NamedTempFile;
use tracing::instrument;

pub(crate) const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) const DEFAULT_MAX_RETRIES: usize = 5;

pub(crate) const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(500); // half a second

pub(crate) fn get(request: &GetRequest) -> Result<GetResponse, GetError> {
    let client = reqwest::blocking::ClientBuilder::new()
        .connect_timeout(request.connect_timeout)
        .use_rustls_tls()
        .build()
        .expect("Should be able to create the HTTP client");

    let retry_strategy = Fixed::from(request.retry_delay).take(request.max_retries);

    retry_with_index(retry_strategy, |index| {
        let attempt = index - 1;

        // fail early if the temporary download file can't be created
        let mut response_file = match tempfile::NamedTempFile::new() {
            Ok(named_temp_file) => named_temp_file,
            Err(e) => return OperationResult::Err(GetError::Write(e)),
        };

        let timer = print::sub_start_timer(if attempt == 0 {
            format!("GET {}", style::url(&request.url))
        } else {
            format!("Retry attempt {attempt} of {}", request.max_retries)
        });

        // helper function to provide a short reason to the timer before returning the error
        let report_error = |timer: GlobalTimer, error: GetError| {
            timer.cancel(error.cancellation_reason());
            if error.retry() {
                OperationResult::Retry(error)
            } else {
                OperationResult::Err(error)
            }
        };

        let mut response = match client
            .get(&request.url)
            .headers(request.headers.clone())
            .send()
            .and_then(Response::error_for_status)
        {
            Ok(response) => response,
            Err(e) => {
                return report_error(timer, GetError::Request(e));
            }
        };

        if let Err(e) = io::copy(&mut response, response_file.as_file_mut()) {
            return report_error(timer, GetError::Write(e));
        }

        if response.status().is_success() {
            timer.done();
        } else if let Some(reason) = response.status().canonical_reason() {
            timer.cancel(reason);
        } else {
            timer.cancel(response.status().as_str());
        }

        OperationResult::Ok(GetResponse {
            response,
            response_file,
        })
    })
    .map_err(|operation| operation.error)
}

#[derive(Debug)]
pub(crate) enum GetError {
    Request(reqwest::Error),
    Write(io::Error),
}

impl GetError {
    fn cancellation_reason(&self) -> String {
        match self {
            GetError::Request(error) => {
                if error.is_connect() {
                    "connection refused".into()
                } else if error.is_timeout() {
                    "timed out".into()
                } else if error.is_builder() {
                    "invalid request".into()
                } else if let Some(status_code) = error.status() {
                    status_code
                        .canonical_reason()
                        .map(ToString::to_string)
                        .unwrap_or(format!("status: {status_code}"))
                } else {
                    error.to_string()
                }
            }
            GetError::Write(_) => "write response error".into(),
        }
    }

    fn retry(&self) -> bool {
        match self {
            GetError::Request(error) => !error.is_builder(),
            GetError::Write(_) => true,
        }
    }
}

impl fmt::Display for GetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetError::Request(e) => write!(f, "{e}"),
            GetError::Write(e) => write!(f, "{e}"),
        }
    }
}

#[derive(Debug, Clone, bon::Builder)]
pub(crate) struct GetRequest {
    #[builder(start_fn, into)]
    url: String,
    #[builder(default = HeaderMap::default())]
    headers: HeaderMap,
    #[builder(default = DEFAULT_CONNECT_TIMEOUT)]
    connect_timeout: Duration,
    #[builder(default = DEFAULT_MAX_RETRIES)]
    max_retries: usize,
    #[builder(default = DEFAULT_RETRY_DELAY)]
    retry_delay: Duration,
}

pub(crate) struct GetResponse {
    response: Response,
    response_file: NamedTempFile,
}

impl GetResponse {
    pub(crate) fn status(&self) -> StatusCode {
        self.response.status()
    }

    pub(crate) fn headers(&self) -> &HeaderMap {
        self.response.headers()
    }

    pub(crate) fn body_as_file(&self) -> io::Result<std::fs::File> {
        self.response_file.reopen()
    }
}

#[instrument(skip_all)]
pub(crate) fn download(download_task: &DownloadTask) -> Result<(), DownloadError> {
    let response = get(&GetRequest::builder(&download_task.source_url)
        .connect_timeout(download_task.connect_timeout)
        .retry_delay(download_task.retry_delay)
        .max_retries(download_task.max_retries)
        .build())
    .map_err(|source| DownloadError::Download {
        url: download_task.source_url.clone(),
        source,
    })?;

    if let Some(ChecksumValidator::Sha256(checksum)) = &download_task.checksum_validator {
        let timer = print::sub_start_timer("Validating");
        match validate_sha256_checksum(&response, checksum, download_task) {
            Ok(()) => timer.done(),
            Err(e) => {
                timer.cancel("error");
                return Err(e);
            }
        }
    }

    match &download_task.extractor {
        Some(Extractor::Gzip(gzip_options)) => {
            let timer = print::sub_start_timer("Extracting");
            match gzip_extract_to_destination(
                &response,
                &download_task.destination,
                gzip_options,
                download_task,
            ) {
                Ok(()) => timer.done(),
                Err(e) => {
                    timer.cancel("error");
                    return Err(e);
                }
            }
        }
        None => {
            let timer = print::sub_start_timer("Saving");
            match save_to_destination(&response, &download_task.destination, download_task) {
                Ok(()) => timer.done(),
                Err(e) => {
                    timer.cancel("error");
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}

fn validate_sha256_checksum(
    response: &GetResponse,
    checksum: &[u8],
    download_task: &DownloadTask,
) -> Result<(), DownloadError> {
    let mut response_body = response
        .body_as_file()
        .map_err(|e| create_write_error(download_task, e))?;
    let mut hasher = Sha256::new();
    io::copy(&mut response_body, &mut hasher).map_err(|e| create_write_error(download_task, e))?;
    let digest = hasher.finalize();
    if checksum == digest.to_vec() {
        Ok(())
    } else {
        Err(DownloadError::ChecksumMismatch {
            url: download_task.source_url.clone(),
            expected_checksum: hex::encode(checksum),
            actual_checksum: format!("{digest:x}"),
        })
    }
}

fn save_to_destination(
    response: &GetResponse,
    destination: &Path,
    download_task: &DownloadTask,
) -> Result<(), DownloadError> {
    let mut response_body = response
        .body_as_file()
        .map_err(|e| create_write_error(download_task, e))?;
    let mut destination_file =
        File::create(destination).map_err(|e| create_write_error(download_task, e))?;
    std::io::copy(&mut response_body, &mut destination_file)
        .map_err(|e| create_write_error(download_task, e))?;
    Ok(())
}

fn gzip_extract_to_destination(
    response: &GetResponse,
    destination_dir: &Path,
    gzip_options: &GzipOptions,
    download_task: &DownloadTask,
) -> Result<(), DownloadError> {
    let gzip_file = response
        .body_as_file()
        .map_err(|e| create_write_error(download_task, e))?;

    let mut archive = tar::Archive::new(MultiGzDecoder::new(gzip_file));

    let entries = archive
        .entries()
        .map_err(|e| create_write_error(download_task, e))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| create_write_error(download_task, e))?;
        let path = entry
            .path()
            .map_err(|e| create_write_error(download_task, e))?;

        // Get the path components
        let path_components: Vec<_> = path.components().collect();

        // Skip if we don't have enough components after stripping
        if path_components.len() <= gzip_options.strip_components {
            continue;
        }

        // Skip if the path contains '..' or is absolute
        if path_components.iter().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        }) {
            continue;
        }

        // Build the stripped path
        let stripped_path: PathBuf = path_components
            .into_iter()
            .skip(gzip_options.strip_components)
            .collect();

        // Skip empty paths
        if stripped_path.components().count() == 0 {
            continue;
        }

        // Skip excluded paths
        let exclude = &gzip_options.exclude;
        if exclude(&stripped_path) {
            continue;
        }

        let entry_type = entry.header().entry_type();
        let dest_path = destination_dir.join(&stripped_path);

        // Create parent directories for regular files and symlinks
        if (entry_type.is_file() || entry_type.is_symlink() || entry_type.is_hard_link())
            && let Some(parent) = dest_path.parent()
        {
            std::fs::create_dir_all(parent).map_err(|e| create_write_error(download_task, e))?;
        }

        entry
            .unpack(dest_path)
            .map_err(|e| create_write_error(download_task, e))?;
    }

    Ok(())
}

fn create_write_error(download_task: &DownloadTask, source: io::Error) -> DownloadError {
    DownloadError::Write {
        url: download_task.source_url.clone(),
        destination: download_task.destination.clone(),
        source,
    }
}

#[derive(Debug, bon::Builder)]
pub(crate) struct DownloadTask<'a> {
    #[builder(start_fn, into)]
    source_url: String,
    #[builder(start_fn, into)]
    destination: PathBuf,
    checksum_validator: Option<ChecksumValidator<'a>>,
    extractor: Option<Extractor>,
    #[builder(default = DEFAULT_CONNECT_TIMEOUT)]
    connect_timeout: Duration,
    #[builder(default = DEFAULT_MAX_RETRIES)]
    max_retries: usize,
    #[builder(default = DEFAULT_RETRY_DELAY)]
    retry_delay: Duration,
}

#[derive(Debug)]
pub(crate) enum Extractor {
    Gzip(GzipOptions),
}

pub(crate) struct GzipOptions {
    pub(crate) strip_components: usize,
    pub(crate) exclude: Box<dyn Fn(&Path) -> bool>,
}

impl core::fmt::Debug for GzipOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GzipOptions")
            .field("strip_components", &self.strip_components)
            .field("exclude", &"<closure>")
            .finish()
    }
}

impl Default for GzipOptions {
    fn default() -> Self {
        Self {
            strip_components: 0,
            exclude: Box::new(|_| false),
        }
    }
}

#[derive(Debug)]
pub(crate) enum ChecksumValidator<'a> {
    Sha256(&'a [u8]),
}

#[derive(Debug)]
pub(crate) enum DownloadError {
    Download {
        url: String,
        source: GetError,
    },
    ChecksumMismatch {
        url: String,
        expected_checksum: String,
        actual_checksum: String,
    },
    Write {
        url: String,
        destination: PathBuf,
        source: io::Error,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use bullet_stream::{global, strip_ansi};
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use indoc::indoc;
    use regex::Regex;
    use std::io::Write;
    use std::{env, fs};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_download_will_not_retry_with_invalid_request() {
        let dst = tempfile::NamedTempFile::new().unwrap();
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(&DownloadTask::builder("htp://bad.url", dst.path()).build()).unwrap_err()
            {
                DownloadError::Download { .. } => {}
                e => panic!("Not the expected error: {e:?}"),
            }
        });
        assert_log_contains_matches(
            &log,
            &[request_failed_matcher("htp://bad.url", "invalid request")],
        );
    }

    #[test]
    fn test_download_success() {
        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(200).set_body_string("test"))
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            download(&DownloadTask::builder(server.uri(), dst.path()).build()).unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[request_success_matcher(server.uri()), saving_matcher()],
        );
        assert_eq!(fs::read_to_string(dst.path()).unwrap(), "test");
    }

    #[test]
    fn test_download_success_after_retry() {
        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(500))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(200).set_body_string("test"))
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            download(&DownloadTask::builder(server.uri(), dst.path()).build()).unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[
                request_failed_matcher(server.uri(), "Internal Server Error"),
                retry_attempt_success_matcher(1, DEFAULT_MAX_RETRIES),
                saving_matcher(),
            ],
        );
        assert_eq!(fs::read_to_string(dst.path()).unwrap(), "test");
    }

    #[test]
    fn test_download_failed_after_retry() {
        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(500))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(400))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(404))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .max_retries(2)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { .. } => {}
                e => panic!("Not the expected error: {e:?}"),
            }
        });

        assert_log_contains_matches(
            &log,
            &[
                request_failed_matcher(server.uri(), "Internal Server Error"),
                retry_attempt_failed_matcher(1, 2, "Bad Request"),
                retry_attempt_failed_matcher(2, 2, "Not Found"),
            ],
        );
    }

    #[test]
    fn test_download_will_not_retry_after_checksum_mismatch() {
        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(200).set_body_string("test"))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        let mut sha256 = Sha256::new();
        sha256.update(b"checksum-mismatch");
        let digest = sha256.finalize().to_vec();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .checksum_validator(ChecksumValidator::Sha256(&digest))
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::ChecksumMismatch { .. } => {}
                e => panic!("Not the expected error: {e:?}"),
            }
        });

        assert_log_contains_matches(
            &log,
            &[
                request_success_matcher(server.uri()),
                validating_failed_matcher(),
            ],
        );
    }

    #[test]
    fn test_download_success_with_checksum_validation() {
        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(200).set_body_string("test"))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        let mut sha256 = Sha256::new();
        sha256.update(b"test");
        let digest = sha256.finalize().to_vec();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .checksum_validator(ChecksumValidator::Sha256(&digest))
                    .build(),
            )
            .unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[
                request_success_matcher(server.uri()),
                validating_matcher(),
                saving_matcher(),
            ],
        );
    }

    #[test]
    fn test_download_will_not_retry_on_file_write_error() {
        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(200).set_body_string("test"))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::tempdir().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(&DownloadTask::builder(server.uri(), dst.path()).build()).unwrap_err() {
                DownloadError::Write { .. } => {}
                e => panic!("Not the expected error {e:?}"),
            }
        });

        assert_log_contains_matches(
            &log,
            &[
                request_success_matcher(server.uri()),
                saving_failed_matcher(),
            ],
        );
    }

    #[test]
    fn test_download_success_with_tar_gz_extraction_strip_and_filter() {
        let tarball = create_archive([
            ("parent/child-a/name.txt", "child-a"),
            ("parent/child-a/grandchild-a/name.txt", "grandchild-a"),
            ("parent/child-b/name.txt", "child-b"),
            ("parent/child-b/grandchild-b/name.txt", "grandchild-b"),
        ]);

        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(fs::read(tarball).unwrap(), "application/gzip"),
                    )
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::tempdir().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .extractor(Extractor::Gzip(GzipOptions {
                        strip_components: 1,
                        exclude: Box::new(|path| {
                            path.components().any(|c| c.as_os_str() == "child-b")
                        }),
                    }))
                    .build(),
            )
            .unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[request_success_matcher(server.uri()), extracting_matcher()],
        );
        assert!(dst.path().join("child-a").is_dir());
        assert!(dst.path().join("child-a/grandchild-a").is_dir());
        assert!(!dst.path().join("child-b").exists());
        assert!(!dst.path().join("child-b/name.txt").exists());
        assert!(!dst.path().join("child-b/grandchild-b").exists());
        assert!(!dst.path().join("child-b/grandchild-b/name.txt").exists());
        assert_eq!(
            fs::read_to_string(dst.path().join("child-a/name.txt")).unwrap(),
            "child-a"
        );
        assert_eq!(
            fs::read_to_string(dst.path().join("child-a/grandchild-a/name.txt")).unwrap(),
            "grandchild-a"
        );
    }

    #[test]
    fn test_download_will_not_retry_with_tar_gz_extraction_error() {
        let tarball = create_archive([("parent/child-a/name.txt", "child-a")]);

        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(fs::read(tarball).unwrap(), "application/gzip"),
                    )
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .extractor(Extractor::Gzip(GzipOptions::default()))
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Write { .. } => {}
                e => panic!("Not the expected error {e:?}"),
            }
        });

        assert_log_contains_matches(
            &log,
            &[
                request_success_matcher(server.uri()),
                extracting_failed_matcher(),
            ],
        );
    }

    #[test]
    fn test_download_success_with_tar_gz_extraction() {
        let tarball = create_archive([
            ("parent/child-a/name.txt", "child-a"),
            ("parent/child-a/grandchild-a/name.txt", "grandchild-a"),
            ("parent/child-b/name.txt", "child-b"),
            ("parent/child-b/grandchild-b/name.txt", "grandchild-b"),
        ]);

        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .set_body_raw(fs::read(tarball).unwrap(), "application/gzip"),
                    )
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::tempdir().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .extractor(Extractor::Gzip(GzipOptions::default()))
                    .build(),
            )
            .unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[request_success_matcher(server.uri()), extracting_matcher()],
        );
        assert!(dst.path().join("parent").is_dir());
        assert!(dst.path().join("parent/child-a").is_dir());
        assert!(dst.path().join("parent/child-a/grandchild-a").is_dir());
        assert!(dst.path().join("parent/child-b").is_dir());
        assert!(dst.path().join("parent/child-b/grandchild-b").is_dir());
        assert_eq!(
            fs::read_to_string(dst.path().join("parent/child-a/name.txt")).unwrap(),
            "child-a"
        );
        assert_eq!(
            fs::read_to_string(dst.path().join("parent/child-a/grandchild-a/name.txt")).unwrap(),
            "grandchild-a"
        );
        assert_eq!(
            fs::read_to_string(dst.path().join("parent/child-b/name.txt")).unwrap(),
            "child-b"
        );
        assert_eq!(
            fs::read_to_string(dst.path().join("parent/child-b/grandchild-b/name.txt")).unwrap(),
            "grandchild-b"
        );
    }

    #[test]
    fn test_client_allows_self_signed_cert() {
        // this is technically not thread-safe but should be okay for now
        // since this is the only test that sets this env var
        #[allow(unsafe_code)]
        unsafe {
            env::remove_var("SSL_CERT_FILE");
        };

        global::with_locked_writer(Vec::<u8>::new(), || {
            assert!(
                get(&GetRequest::builder("https://self-signed.badssl.com")
                    .max_retries(0)
                    .build())
                .is_err()
            );
        });

        let badssl_self_signed_cert_dir = tempfile::tempdir().unwrap();
        let badssl_self_signed_cert = badssl_self_signed_cert_dir
            .path()
            .join("badssl_self_signed_cert.pem");

        // https://github.com/rustls/rustls-native-certs/blob/main/tests/badssl-com-chain.pem
        fs::write(
            &badssl_self_signed_cert,
            indoc! { "
            -----BEGIN CERTIFICATE-----
            MIIDeTCCAmGgAwIBAgIJAMnA8BB8xT6wMA0GCSqGSIb3DQEBCwUAMGIxCzAJBgNV
            BAYTAlVTMRMwEQYDVQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1TYW4gRnJhbmNp
            c2NvMQ8wDQYDVQQKDAZCYWRTU0wxFTATBgNVBAMMDCouYmFkc3NsLmNvbTAeFw0y
            MTEwMTEyMDAzNTRaFw0yMzEwMTEyMDAzNTRaMGIxCzAJBgNVBAYTAlVTMRMwEQYD
            VQQIDApDYWxpZm9ybmlhMRYwFAYDVQQHDA1TYW4gRnJhbmNpc2NvMQ8wDQYDVQQK
            DAZCYWRTU0wxFTATBgNVBAMMDCouYmFkc3NsLmNvbTCCASIwDQYJKoZIhvcNAQEB
            BQADggEPADCCAQoCggEBAMIE7PiM7gTCs9hQ1XBYzJMY61yoaEmwIrX5lZ6xKyx2
            PmzAS2BMTOqytMAPgLaw+XLJhgL5XEFdEyt/ccRLvOmULlA3pmccYYz2QULFRtMW
            hyefdOsKnRFSJiFzbIRMeVXk0WvoBj1IFVKtsyjbqv9u/2CVSndrOfEk0TG23U3A
            xPxTuW1CrbV8/q71FdIzSOciccfCFHpsKOo3St/qbLVytH5aohbcabFXRNsKEqve
            ww9HdFxBIuGa+RuT5q0iBikusbpJHAwnnqP7i/dAcgCskgjZjFeEU4EFy+b+a1SY
            QCeFxxC7c3DvaRhBB0VVfPlkPz0sw6l865MaTIbRyoUCAwEAAaMyMDAwCQYDVR0T
            BAIwADAjBgNVHREEHDAaggwqLmJhZHNzbC5jb22CCmJhZHNzbC5jb20wDQYJKoZI
            hvcNAQELBQADggEBAC4DensZ5tCTeCNJbHABYPwwqLUFOMITKOOgF3t8EqOan0CH
            ST1NNi4jPslWrVhQ4Y3UbAhRBdqXl5N/NFfMzDosPpOjFgtifh8Z2s3w8vdlEZzf
            A4mYTC8APgdpWyNgMsp8cdXQF7QOfdnqOfdnY+pfc8a8joObR7HEaeVxhJs+XL4E
            CLByw5FR+svkYgCbQGWIgrM1cRpmXemt6Gf/XgFNP2PdubxqDEcnWlTMk8FCBVb1
            nVDSiPjYShwnWsOOshshCRCAiIBPCKPX0QwKDComQlRrgMIvddaSzFFTKPoNZjC+
            CUspSNnL7V9IIHvqKlRSmu+zIpm2VJCp1xLulk8=
            -----END CERTIFICATE-----
        "},
        )
        .unwrap();

        #[allow(unsafe_code)]
        unsafe {
            env::set_var("SSL_CERT_FILE", badssl_self_signed_cert);
        };

        global::with_locked_writer(Vec::<u8>::new(), || {
            assert!(
                get(&GetRequest::builder("https://self-signed.badssl.com")
                    .max_retries(0)
                    .build())
                .is_ok()
            );
        });

        #[allow(unsafe_code)]
        unsafe {
            env::remove_var("SSL_CERT_FILE");
        };
    }

    fn assert_log_contains_matches(log: &[u8], matchers: &[Regex]) {
        let output = strip_ansi(String::from_utf8_lossy(log));
        let actual_lines = output.lines().map(str::trim).collect::<Vec<_>>();
        assert_eq!(
            matchers.len(),
            actual_lines.len(),
            "Expected matchers does not match length of actual logged lines\nMatchers:\n{matchers:?}\nLines: {actual_lines:?}"
        );
        actual_lines
            .iter()
            .zip(matchers.iter())
            .for_each(|(actual, matcher)| {
                assert!(
                    matcher.is_match(actual),
                    "Expected matcher did not match line\nMatcher: {matcher:?}\nLine: {actual}"
                );
            });
    }

    const PROGRESS_DOTS: &str = r"\.+";

    const TIMER: &str = r"\(<?\s?\d+\.\d+s\)";

    fn request_success_matcher(url: impl AsRef<str>) -> Regex {
        let url = url.as_ref().replace('.', r"\.");
        Regex::new(&format!(r"- GET {url}\/? {PROGRESS_DOTS} {TIMER}")).unwrap()
    }

    fn retry_attempt_success_matcher(attempt: u32, max_retries: usize) -> Regex {
        Regex::new(&format!(
            r"- Retry attempt {attempt} of {max_retries} {PROGRESS_DOTS} {TIMER}"
        ))
        .unwrap()
    }

    fn request_failed_matcher(url: impl AsRef<str>, reason: &str) -> Regex {
        let url = url.as_ref().replace('.', r"\.");
        Regex::new(&format!(r"- GET {url}\/? {PROGRESS_DOTS} \({reason}\)")).unwrap()
    }

    fn retry_attempt_failed_matcher(attempt: u32, max_retries: u32, reason: &str) -> Regex {
        Regex::new(&format!(
            r"- Retry attempt {attempt} of {max_retries} {PROGRESS_DOTS} \({reason}\)"
        ))
        .unwrap()
    }

    fn validating_matcher() -> Regex {
        Regex::new(&format!(r"- Validating {PROGRESS_DOTS} {TIMER}")).unwrap()
    }

    fn validating_failed_matcher() -> Regex {
        Regex::new(&format!(r"- Validating {PROGRESS_DOTS} \(error\)")).unwrap()
    }

    fn saving_matcher() -> Regex {
        Regex::new(&format!(r"- Saving {PROGRESS_DOTS} {TIMER}")).unwrap()
    }

    fn saving_failed_matcher() -> Regex {
        Regex::new(&format!(r"- Saving {PROGRESS_DOTS} \(error\)")).unwrap()
    }

    fn extracting_matcher() -> Regex {
        Regex::new(&format!(r"- Extracting {PROGRESS_DOTS} {TIMER}")).unwrap()
    }

    fn extracting_failed_matcher() -> Regex {
        Regex::new(&format!(r"- Extracting {PROGRESS_DOTS} \(error\)")).unwrap()
    }

    fn create_archive<'a>(
        files: impl IntoIterator<Item = (impl Into<PathBuf>, &'a str)>,
    ) -> PathBuf {
        let archive_folder = tempfile::tempdir().unwrap();
        for (path, contents) in files {
            let path = path.into();
            assert!(!path.is_absolute(), "Only relative paths allowed");
            let archive_path = archive_folder.path().join(path);
            fs::create_dir_all(archive_path.parent().unwrap()).unwrap();
            fs::write(archive_path, contents).unwrap();
        }

        let (archive_file, archive_path) = tempfile::NamedTempFile::new().unwrap().keep().unwrap();
        let mut tarball = tar::Builder::new(GzEncoder::new(archive_file, Compression::default()));
        tarball.append_dir_all("", archive_folder.path()).unwrap();
        // for whatever reason, `tar.finish()` was producing an invalid gzip file, but using `tar.into_inner()`
        // to get the underlying gzip encoder and flushing+finishing that writer works
        let mut gzip = tarball.into_inner().unwrap();
        gzip.flush().unwrap();
        gzip.finish().unwrap();

        archive_path
    }
}
