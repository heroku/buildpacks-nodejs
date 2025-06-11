use bullet_stream::style;
use reqwest::{IntoUrl, Request, Response};
use reqwest_middleware::{ClientBuilder, Middleware, Next};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;
use ureq::http::Extensions;

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(10);

const DEFAULT_RETRIES: u32 = 5;

#[bon::builder]
pub fn download_file_sync(
    from_url: impl IntoUrl + std::fmt::Display + Clone,
    to_file: impl AsRef<Path>,
    #[builder(into)] downloading_message: Option<String>,
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
            .maybe_downloading_message(downloading_message)
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
    #[builder(into)] downloading_message: Option<String>,
    #[builder(default = DEFAULT_CONNECT_TIMEOUT)] connect_timeout: Duration,
    #[builder(default = DEFAULT_READ_TIMEOUT)] read_timeout: Duration,
    #[builder(default = DEFAULT_RETRIES)] max_retries: u32,
) -> Result<(), DownloadError> {
    let to_file = to_file.as_ref();
    let mut output_file = tokio::fs::File::options()
        .write(true)
        .create(true)
        .truncate(true)
        .open(to_file)
        .await
        .map_err(|e| DownloadError::OpenFile(to_file.to_path_buf(), from_url.to_string(), e))?;

    download_writer()
        .from_url(from_url)
        .writer(&mut output_file)
        .maybe_downloading_message(downloading_message)
        .connect_timeout(connect_timeout)
        .read_timeout(read_timeout)
        .max_retries(max_retries)
        .call()
        .await
}

#[bon::builder]
pub async fn download_writer<W>(
    from_url: impl IntoUrl + std::fmt::Display + Clone,
    writer: &mut W,
    #[builder(into)] downloading_message: Option<String>,
    #[builder(default = DEFAULT_CONNECT_TIMEOUT)] connect_timeout: Duration,
    #[builder(default = DEFAULT_READ_TIMEOUT)] read_timeout: Duration,
    #[builder(default = DEFAULT_RETRIES)] max_retries: u32,
) -> Result<(), DownloadError>
where
    W: tokio::io::AsyncWrite + Unpin + ?Sized,
{
    let downloading_message = downloading_message
        .unwrap_or_else(|| format!("Downloading {}", style::url(from_url.to_string())));

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
    .with(RetryLoggingMiddleware::new(
        max_retries,
        downloading_message,
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

    tokio::io::copy(&mut response_data.as_slice(), writer)
        .await
        .expect("Real error handling in the future");

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

struct RetryLoggingMiddleware {
    count: AtomicU32,
    initial_message: String,
    max_retries: u32,
}

impl RetryLoggingMiddleware {
    fn new(max_retries: u32, downloading_message: String) -> Self {
        Self {
            count: AtomicU32::new(0),
            initial_message: downloading_message,
            max_retries,
        }
    }
}

#[async_trait::async_trait]
impl Middleware for RetryLoggingMiddleware {
    async fn handle(
        &self,
        req: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        // increment and acquire the previous value
        let previous_value = self.count.fetch_add(1, SeqCst);
        let message = if previous_value == 0 {
            self.initial_message.clone()
        } else {
            format!("Retry attempt {previous_value} of {}", self.max_retries)
        };
        let timer = bullet_stream::global::print::sub_start_timer(message);
        let response = next.run(req, extensions).await;
        match &response {
            Ok(response) => {
                if response.status().is_success() {
                    let _ = timer.done();
                } else {
                    let _ = timer.cancel("unexpected response");
                }
            }
            Err(e) => {
                let reason = if e.is_connect() {
                    "connection refused".into()
                } else if e.is_timeout() {
                    "timed out".into()
                } else {
                    e.to_string()
                };
                let _ = timer.cancel(reason);
            }
        }
        response
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use bullet_stream::{global, strip_ansi};
    use regex::Regex;
    use reqwest::StatusCode;
    use std::cell::RefCell;
    use std::future::Future;
    use std::io::Write;
    use std::ops::{Div, Mul};
    use std::{fs, net};
    use tempfile::NamedTempFile;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

    #[tokio::test]
    async fn test_download_success_no_retry() {
        global::set_writer(TestLogger);

        let mock_download = setup_mock_download()
            .respond_with_success_after_n_requests(1)
            .call()
            .await;

        let dst = NamedTempFile::new().unwrap();

        download_file()
            .from_url(mock_download.url())
            .to_file(dst.path())
            .call()
            .await
            .unwrap();

        assert_download_contents(&dst);
        assert_log_contains_matches(&[download_success_matcher(mock_download.url())]);
    }

    #[tokio::test]
    async fn test_download_success_after_retry() {
        global::set_writer(TestLogger);

        let mock_download = setup_mock_download()
            .respond_with_success_after_n_requests(2)
            .call()
            .await;

        let dst = NamedTempFile::new().unwrap();

        download_file()
            .from_url(mock_download.url())
            .to_file(dst.path())
            .max_retries(2)
            .call()
            .await
            .unwrap();

        assert_download_contents(&dst);

        assert_log_contains_matches(&[
            download_failed_matcher(mock_download.url(), "unexpected response"),
            retry_attempt_success_matcher(1, 2),
        ]);
    }

    #[tokio::test]
    async fn test_download_failed_after_retry() {
        global::set_writer(TestLogger);

        let mock_download = setup_mock_download()
            .respond_with_success_after_n_requests(4)
            // because we'll only ever get to 3 (first request + 2 retries)
            .expected_responses(3)
            .call()
            .await;

        let dst = NamedTempFile::new().unwrap();

        let error = download_file()
            .from_url(mock_download.url())
            .to_file(dst.path())
            .max_retries(2)
            .call()
            .await
            .unwrap_err();

        match error {
            DownloadError::Request(_, _) => {
                assert_no_download_contents(&dst);

                assert_log_contains_matches(&[
                    download_failed_matcher(mock_download.url(), "unexpected response"),
                    retry_attempt_failed_matcher(1, 2, "unexpected response"),
                    retry_attempt_failed_matcher(2, 2, "unexpected response"),
                ]);
            }
            _ => panic!("Unexpected error: {error:?}"),
        }
    }

    #[tokio::test]
    async fn test_download_retry_connect_timeout() {
        global::set_writer(TestLogger);

        // let respond_with_success_after_n_requests = 1;
        let connect_delay = Duration::from_secs(1);
        let connect_timeout = connect_delay.div(2);
        let read_timeout = connect_delay.mul(2);

        // Borrowed this url from the reqwest test for connect timeouts
        let download_url = "http://192.0.2.1:81/slow";

        let dst = NamedTempFile::new().unwrap();

        let error = download_file()
            .from_url(download_url)
            .to_file(dst.path())
            .max_retries(2)
            .connect_timeout(connect_timeout)
            .read_timeout(read_timeout)
            .call()
            .await
            .unwrap_err();

        match error {
            DownloadError::Request(_, _) => {
                assert_no_download_contents(&dst);

                assert_log_contains_matches(&[
                    download_failed_matcher(download_url, "connection refused"),
                    retry_attempt_failed_matcher(1, 2, "connection refused"),
                    retry_attempt_failed_matcher(2, 2, "connection refused"),
                ]);
            }
            _ => panic!("Unexpected error: {error:?}"),
        }
    }

    #[tokio::test]
    async fn test_download_retry_response_timeout() {
        global::set_writer(TestLogger);

        let read_timeout = Duration::from_millis(100);

        let server = test_server(move |_req| {
            async {
                // delay returning the response
                tokio::time::sleep(Duration::from_millis(300)).await;
                http::Response::default()
            }
        });

        let dst = NamedTempFile::new().unwrap();

        let error = download_file()
            .from_url(server.uri())
            .to_file(dst.path())
            .max_retries(2)
            .read_timeout(read_timeout)
            .call()
            .await
            .unwrap_err();

        match error {
            DownloadError::Request(_, _) => {
                assert_no_download_contents(&dst);

                assert_log_contains_matches(&[
                    download_failed_matcher(server.uri(), "timed out"),
                    retry_attempt_failed_matcher(1, 2, "timed out"),
                    retry_attempt_failed_matcher(2, 2, "timed out"),
                ]);
            }
            _ => panic!("Unexpected error: {error:?}"),
        }
    }

    struct TestServer {
        addr: net::SocketAddr,
        panic_rx: std::sync::mpsc::Receiver<()>,
        shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    }

    impl TestServer {
        fn uri(&self) -> String {
            format!("http://{}/", self.addr)
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(tx) = self.shutdown_tx.take() {
                let _ = tx.send(());
            }

            if !::std::thread::panicking() {
                self.panic_rx
                    .recv_timeout(Duration::from_secs(3))
                    .expect("test server should not panic");
            }
        }
    }

    // matches reqwest test server to help simulate read response timeouts
    fn test_server<F, Fut>(func: F) -> TestServer
    where
        F: Fn(http::Request<hyper::body::Incoming>) -> Fut + Clone + Send + 'static,
        Fut: Future<Output = http::Response<reqwest::Body>> + Send + 'static,
    {
        let test_name = std::thread::current()
            .name()
            .unwrap_or("<unknown>")
            .to_string();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("new rt");
            let listener = rt.block_on(async move {
                tokio::net::TcpListener::bind(&std::net::SocketAddr::from(([127, 0, 0, 1], 0)))
                    .await
                    .unwrap()
            });
            let addr = listener.local_addr().unwrap();

            let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
            let (panic_tx, panic_rx) = std::sync::mpsc::channel();
            let tname = format!("test({test_name})-support-server");
            std::thread::Builder::new()
                .name(tname)
                .spawn(move || {
                    rt.block_on(async move {
                        let builder =
                            hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new());

                        loop {
                            tokio::select! {
                            _ = &mut shutdown_rx => {
                                break;
                            }
                            accepted = listener.accept() => {
                                let (io, _) = accepted.expect("accepted");
                                let func = func.clone();
                                let svc = hyper::service::service_fn(move |req| {
                                    let fut = func(req);
                                    async move { Ok::<_, std::convert::Infallible>(fut.await) }
                                });
                                let builder = builder.clone();
                                tokio::spawn(async move {
                                    let _ = builder.serve_connection_with_upgrades(hyper_util::rt::TokioIo::new(io), svc).await;
                                });
                            }
                        }
                        }
                        let _ = panic_tx.send(());
                    });
                })
                .expect("thread spawn");

            TestServer {
                addr,
                panic_rx,
                shutdown_tx: Some(shutdown_tx),
            }
        })
            .join()
            .unwrap()
    }

    thread_local! {
        static THREAD_LOCAL_WRITER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    struct TestLogger;

    impl TestLogger {
        fn take() -> Vec<u8> {
            THREAD_LOCAL_WRITER.take()
        }
    }

    impl Write for TestLogger {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            THREAD_LOCAL_WRITER.with_borrow_mut(|writer| writer.write(buf))
        }

        fn flush(&mut self) -> io::Result<()> {
            THREAD_LOCAL_WRITER.with_borrow_mut(std::io::Write::flush)
        }
    }

    #[bon::builder]
    async fn setup_mock_download(
        respond_with_success_after_n_requests: u32,
        expected_responses: Option<u32>,
    ) -> MockDownload {
        let server = MockServer::start().await;
        let expected_responses =
            expected_responses.unwrap_or(respond_with_success_after_n_requests);
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(RetryResponder::new(respond_with_success_after_n_requests))
            .expect(u64::from(expected_responses))
            .mount(&server)
            .await;
        MockDownload { server }
    }

    struct MockDownload {
        server: MockServer,
    }

    impl MockDownload {
        pub(crate) fn url(&self) -> String {
            self.server.uri()
        }
    }

    struct RetryResponder {
        requests_attempted: AtomicU32,
        respond_with_success_after_n_requests: u32,
    }

    impl RetryResponder {
        fn new(respond_with_success_after_n_requests: u32) -> Self {
            Self {
                requests_attempted: AtomicU32::new(0),
                respond_with_success_after_n_requests,
            }
        }
    }

    impl Respond for RetryResponder {
        fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
            let requests_attempted = self.requests_attempted.fetch_add(1, SeqCst) + 1;

            if requests_attempted >= self.respond_with_success_after_n_requests {
                ResponseTemplate::new(StatusCode::OK).set_body_string(TEST_REQUEST_CONTENTS)
            } else {
                ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    const TEST_REQUEST_CONTENTS: &str = "test request";

    fn assert_download_contents(download_file: &NamedTempFile) {
        assert_eq!(
            fs::read_to_string(download_file.path()).unwrap(),
            TEST_REQUEST_CONTENTS
        );
    }

    fn assert_no_download_contents(download_file: &NamedTempFile) {
        assert_eq!(download_file.as_file().metadata().unwrap().len(), 0);
    }

    fn assert_log_contains_matches(matchers: &[Regex]) {
        let output = strip_ansi(String::from_utf8_lossy(&TestLogger::take()));
        let actual_lines = output.lines().map(str::trim).collect::<Vec<_>>();
        assert_eq!(matchers.len(), actual_lines.len(), "Expected matchers does not match length of actual logged lines\nMatchers:\n{matchers:?}\nLines: {actual_lines:?}");
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

    const PROGRESS_DOTS: &str = r"(?:\s\.+\s)?";
    const TIMER: &str = r"\(< \d+\.\d+s\)";

    fn download_success_matcher(url: impl AsRef<str>) -> Regex {
        let url = url.as_ref();
        Regex::new(&format!(r"- Downloading {url}{PROGRESS_DOTS}{TIMER}")).unwrap()
    }

    fn download_failed_matcher(url: impl AsRef<str>, reason: &str) -> Regex {
        let url = url.as_ref();
        Regex::new(&format!(r"- Downloading {url}{PROGRESS_DOTS}\({reason}\)")).unwrap()
    }

    fn retry_attempt_success_matcher(attempt: u32, max_retries: u32) -> Regex {
        Regex::new(&format!(
            r"- Retry attempt {attempt} of {max_retries}{PROGRESS_DOTS}{TIMER}"
        ))
        .unwrap()
    }

    fn retry_attempt_failed_matcher(attempt: u32, max_retries: u32, reason: &str) -> Regex {
        Regex::new(&format!(
            r"- Retry attempt {attempt} of {max_retries}{PROGRESS_DOTS}\({reason}\)"
        ))
        .unwrap()
    }
}
