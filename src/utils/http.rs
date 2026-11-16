use crate::o11y::{HTTP_ATTEMPTS, HTTP_TRANSFER_BYTES_PER_SECOND, HTTP_TRANSFER_LONGEST_SLOW_MS};
use bullet_stream::global::print;
use bullet_stream::style;
use digest_io::IoWrapper;
use flate2::read::MultiGzDecoder;
use http::{HeaderMap, StatusCode};
use reqwest::blocking::Response;
use retry::delay::{Exponential, Fixed};
use retry::{OperationResult, retry_with_index};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fmt;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::NamedTempFile;
use tracing::field::Empty;
use tracing::{Span, instrument};

// The defaults mirror the curl options proposed for the classic Node.js buildpack in
// https://github.com/heroku/heroku-buildpack-nodejs/pull/1853.

/// curl `--connect-timeout`
pub(crate) const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// curl `--max-time`
pub(crate) const DEFAULT_MAX_TIME: Duration = Duration::from_secs(120);

/// curl `--retry`
pub(crate) const DEFAULT_RETRY: usize = 5;

/// Without `--retry-delay`, curl waits 1s before the first retry and doubles the delay for each
/// retry after that, up to this cap.
const MAX_RETRY_BACKOFF: Duration = Duration::from_secs(600);

/// curl `--retry-max-time`
pub(crate) const DEFAULT_RETRY_MAX_TIME: Duration = Duration::from_secs(120);

/// curl `--speed-limit` (bytes per second)
pub(crate) const DEFAULT_SPEED_LIMIT: u64 = 512 * 1024;

/// curl `--speed-time`
pub(crate) const DEFAULT_SPEED_TIME: Duration = Duration::from_secs(15);

pub(crate) fn get(request: &GetRequest) -> Result<GetResponse, GetError> {
    let start = Instant::now();
    let retry_deadline = start + request.retry_max_time;

    // A span for the whole request, with a child span for each attempt.
    let url = telemetry_url(&request.url);
    let get_span = tracing::info_span!(
        "http_get",
        url.full = url.as_ref().map(reqwest::Url::as_str),
        { HTTP_ATTEMPTS } = Empty,
        otel.status_code = Empty,
    );
    let _get_guard = get_span.enter();

    // The most recent response received across all attempts, attached to any error.
    let mut last_response: Option<Box<LastResponse>> = None;
    let mut attempts: Vec<FailedAttempt> = Vec::new();

    // The blocking client applies `timeout` to sending the request and to each read of the response
    // body separately. Bounding both by `speed_time` fails a transfer that makes no progress at all
    // for that long, which `TransferReader` can't detect because the read never returns.
    let client = reqwest::blocking::ClientBuilder::new()
        .connect_timeout(request.connect_timeout)
        .timeout(request.speed_time)
        .use_rustls_tls()
        .build()
        .expect("Should be able to create the HTTP client");

    let retry_delays = retry_delays(request);

    retry_with_index(retry_delays.clone(), |index| {
        let attempt = index - 1;
        let attempt_start = Instant::now();
        let attempt_span = attempt_span(url.as_ref());
        let _attempt_guard = attempt_span.enter();

        // fail early if the temporary download file can't be created
        let mut response_file = match tempfile::NamedTempFile::new() {
            Ok(named_temp_file) => named_temp_file,
            Err(source) => {
                let error = AttemptError::Write(source);
                record_attempt_error(&attempt_span, &error);
                attempts.push(FailedAttempt {
                    error,
                    duration: attempt_start.elapsed(),
                });
                return OperationResult::Err(StopReason::NotRetryable);
            }
        };

        let timer = print::sub_start_timer(if attempt == 0 {
            format!("GET {}", style::url(&request.url))
        } else {
            format!("Retry attempt {attempt} of {}", request.retry)
        });

        match get_attempt(
            &client,
            request,
            &mut response_file,
            &mut last_response,
            &attempt_span,
        ) {
            Ok(response) => {
                get_span.record(HTTP_ATTEMPTS, as_i64(index));
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
            }
            Err(error) => {
                timer.cancel(error.cancellation_reason());
                record_attempt_error(&attempt_span, &error);
                // `retry_delays[attempt]` is the delay before the next attempt
                let next_retry_delay = usize::try_from(attempt)
                    .ok()
                    .and_then(|attempt| retry_delays.get(attempt));
                let stop_reason = match next_retry_delay {
                    _ if !error.retry() => Some(StopReason::NotRetryable),
                    None => Some(StopReason::RetriesExhausted),
                    // like curl, don't start a retry that would begin after `retry_max_time`
                    Some(delay) if Instant::now() + *delay >= retry_deadline => {
                        Some(StopReason::RetryMaxTime)
                    }
                    Some(_) => None,
                };
                attempts.push(FailedAttempt {
                    error,
                    duration: attempt_start.elapsed(),
                });
                match stop_reason {
                    Some(stop_reason) => OperationResult::Err(stop_reason),
                    None => OperationResult::Retry(StopReason::RetriesExhausted),
                }
            }
        }
    })
    .map_err(|operation| {
        get_span.record(HTTP_ATTEMPTS, as_i64(attempts.len()));
        get_span.record("otel.status_code", "ERROR");
        GetError {
            attempts,
            stop_reason: operation.error,
            elapsed: start.elapsed(),
            last_response,
        }
    })
}

/// A span for a single attempt, with the fields `reqwest-tracing` recorded for each request.
fn attempt_span(url: Option<&reqwest::Url>) -> Span {
    tracing::info_span!(
        "GET",
        otel.name = "GET",
        otel.kind = "client",
        http.request.method = "GET",
        url.full = url.map(reqwest::Url::as_str),
        url.scheme = url.map(reqwest::Url::scheme),
        server.address = url.and_then(reqwest::Url::host_str),
        server.port = url
            .and_then(reqwest::Url::port_or_known_default)
            .map(i64::from),
        http.response.status_code = Empty,
        { HTTP_TRANSFER_BYTES_PER_SECOND } = Empty,
        { HTTP_TRANSFER_LONGEST_SLOW_MS } = Empty,
        otel.status_code = Empty,
        error.message = Empty,
    )
}

/// The request URL with any credentials removed, as `reqwest-tracing` recorded it.
fn telemetry_url(url: &str) -> Option<reqwest::Url> {
    let mut url = reqwest::Url::parse(url).ok()?;
    let _ = url.set_username("");
    let _ = url.set_password(None);
    Some(url)
}

/// `tracing-opentelemetry` only exports `i64` as a number (unsigned values become strings), so
/// numeric fields need to be recorded as `i64` to be queried as numbers.
fn as_i64(value: impl TryInto<i64>) -> i64 {
    value.try_into().unwrap_or(i64::MAX)
}

fn record_attempt_error(span: &Span, error: &AttemptError) {
    span.record("otel.status_code", "ERROR");
    span.record("error.message", error.to_string());
}

/// The delay before each retry. Like curl `--retry-delay`, a fixed delay replaces the default
/// exponential backoff.
fn retry_delays(request: &GetRequest) -> Vec<Duration> {
    match request.retry_delay {
        Some(delay) => Fixed::from(delay).take(request.retry).collect(),
        None => Exponential::from_millis(1000)
            .map(|delay| delay.min(MAX_RETRY_BACKOFF))
            .take(request.retry)
            .collect(),
    }
}

/// A single request, with the response body written to `response_file`.
fn get_attempt(
    client: &reqwest::blocking::Client,
    request: &GetRequest,
    response_file: &mut NamedTempFile,
    last_response: &mut Option<Box<LastResponse>>,
    span: &Span,
) -> Result<Response, AttemptError> {
    let max_time_deadline = Instant::now() + request.max_time;

    let response = client
        .get(&request.url)
        .headers(request.headers.clone())
        .send()
        .map_err(|error| {
            // reqwest reports errors building the request (e.g. an invalid URL) when it's sent
            if error.is_builder() {
                AttemptError::InvalidRequest(error)
            } else {
                AttemptError::Request(error)
            }
        })?;

    span.record(
        "http.response.status_code",
        i64::from(response.status().as_u16()),
    );

    *last_response = Some(Box::new(LastResponse {
        status: response.status(),
        headers: response.headers().clone(),
    }));

    let mut response = response.error_for_status().map_err(AttemptError::Request)?;

    let mut body = TransferReader::new(&mut response, max_time_deadline, request);
    let copy_result = io::copy(&mut body, response_file.as_file_mut());
    body.record_speed(span);
    copy_result.map_err(copy_error)?;

    Ok(response)
}

/// Errors reading the response body arrive as I/O errors wrapping a [`TransferTimeout`] or the
/// reqwest error. Anything else is a failure writing the temporary download file.
fn copy_error(error: io::Error) -> AttemptError {
    match error.downcast::<TransferTimeout>() {
        Ok(timeout) => AttemptError::Timeout(timeout),
        Err(error) => match error.downcast::<reqwest::Error>() {
            Ok(source) => AttemptError::Request(source),
            Err(error) => AttemptError::Write(error),
        },
    }
}

/// curl measures the current transfer speed as an average over about the last 5 seconds.
const SPEED_WINDOW: Duration = Duration::from_secs(5);

/// Enforces curl's `--max-time` and `--speed-limit`/`--speed-time` on the response body.
///
/// Like curl, the transfer fails once the current speed has stayed below `speed_limit` for
/// `speed_time`, and any moment back above the limit resets that. Both checks only run when a read
/// returns. A read that gets no data at all is bounded by the client's `speed_time` timeout, which
/// is reported the same way.
struct TransferReader<R> {
    inner: R,
    max_time: Duration,
    max_time_deadline: Instant,
    speed_limit: u64,
    speed_time: Duration,
    // the current speed is averaged over this window, shortened for a `speed_time` below 5s
    speed_window: Duration,
    start: Instant,
    total_bytes: u64,
    // `(time, total_bytes)` taken about every `speed_window / 5`, spanning the last `speed_window`
    samples: VecDeque<(Instant, u64)>,
    // when the current speed first dropped below `speed_limit`
    slow_since: Option<Instant>,
    // the longest the current speed stayed below `speed_limit`, for telemetry
    longest_slow: Duration,
}

impl<R> TransferReader<R> {
    fn new(inner: R, max_time_deadline: Instant, request: &GetRequest) -> Self {
        let start = Instant::now();
        Self {
            inner,
            max_time: request.max_time,
            max_time_deadline,
            speed_limit: request.speed_limit,
            speed_time: request.speed_time,
            speed_window: SPEED_WINDOW.min(request.speed_time),
            start,
            total_bytes: 0,
            samples: VecDeque::from([(start, 0)]),
            slow_since: None,
            longest_slow: Duration::ZERO,
        }
    }

    fn timeout(&self) -> TransferTimeout {
        if Instant::now() >= self.max_time_deadline {
            TransferTimeout::MaxTime(self.max_time)
        } else {
            TransferTimeout::SpeedLimit {
                speed_limit: self.speed_limit,
                speed_time: self.speed_time,
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn below_speed_limit(&mut self, now: Instant) -> bool {
        let sample_interval = self.speed_window / 5;
        if self
            .samples
            .back()
            .is_none_or(|(time, _)| now - *time >= sample_interval)
        {
            self.samples.push_back((now, self.total_bytes));
        }
        // keep the newest sample that's at least `speed_window` old as the start of the average
        while self.samples.len() > 1 && now - self.samples[1].0 >= self.speed_window {
            self.samples.pop_front();
        }

        let (start, start_bytes) = self.samples[0];
        let elapsed = now - start;
        // too soon to measure a speed
        if elapsed < sample_interval {
            return false;
        }
        let speed = (self.total_bytes - start_bytes) as f64 / elapsed.as_secs_f64();
        speed < self.speed_limit as f64
    }

    /// Records how fast the body transferred, so the speed limit can be checked against real
    /// downloads.
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn record_speed(&self, span: &Span) {
        let elapsed = self.start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            span.record(
                HTTP_TRANSFER_BYTES_PER_SECOND,
                (self.total_bytes as f64 / elapsed) as i64,
            );
        }
        let longest_slow = match self.slow_since {
            Some(slow_since) => self.longest_slow.max(slow_since.elapsed()),
            None => self.longest_slow,
        };
        span.record(
            HTTP_TRANSFER_LONGEST_SLOW_MS,
            as_i64(longest_slow.as_millis()),
        );
    }
}

impl<R: io::Read> io::Read for TransferReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if Instant::now() >= self.max_time_deadline {
            return Err(TransferTimeout::MaxTime(self.max_time).into());
        }

        let n = match self.inner.read(buf) {
            Ok(n) => n,
            // the client's per-read timeout, so no data arrived for `speed_time`
            Err(error) if is_reqwest_timeout(&error) => return Err(self.timeout().into()),
            Err(error) => return Err(error),
        };

        // the transfer is complete, so there's no speed to enforce
        if n == 0 {
            return Ok(n);
        }

        self.total_bytes += n as u64;
        let now = Instant::now();
        if self.below_speed_limit(now) {
            let slow_since = *self.slow_since.get_or_insert(now);
            self.longest_slow = self.longest_slow.max(now - slow_since);
            if now - slow_since >= self.speed_time {
                return Err(self.timeout().into());
            }
        } else {
            self.slow_since = None;
        }

        Ok(n)
    }
}

/// Failures reading the response body are I/O errors wrapping the reqwest error.
fn is_reqwest_timeout(error: &io::Error) -> bool {
    error
        .get_ref()
        .and_then(|inner| inner.downcast_ref::<reqwest::Error>())
        .is_some_and(reqwest::Error::is_timeout)
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum TransferTimeout {
    MaxTime(Duration),
    SpeedLimit {
        speed_limit: u64,
        speed_time: Duration,
    },
}

impl fmt::Display for TransferTimeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransferTimeout::MaxTime(max_time) => {
                write!(f, "Operation timed out after {}s", max_time.as_secs_f32())
            }
            TransferTimeout::SpeedLimit {
                speed_limit,
                speed_time,
            } => write!(
                f,
                "Operation too slow. Less than {speed_limit} bytes/sec transferred the last {}s",
                speed_time.as_secs_f32()
            ),
        }
    }
}

impl std::error::Error for TransferTimeout {}

impl From<TransferTimeout> for io::Error {
    fn from(timeout: TransferTimeout) -> Self {
        io::Error::new(io::ErrorKind::TimedOut, timeout)
    }
}

/// The error from a single attempt of [`get`].
#[derive(Debug)]
pub(crate) enum AttemptError {
    /// The request couldn't be built, e.g. because the URL is invalid.
    InvalidRequest(reqwest::Error),
    /// Sending the request, an HTTP error status, or reading the response body failed.
    Request(reqwest::Error),
    /// Creating or writing the temporary download file failed.
    Write(io::Error),
    /// The response body exceeded `max_time` or was slower than the speed limit.
    Timeout(TransferTimeout),
}

impl AttemptError {
    fn cancellation_reason(&self) -> String {
        match self {
            AttemptError::InvalidRequest(_) => "invalid request".into(),
            AttemptError::Request(source) => request_error_reason(source),
            AttemptError::Write(_) => "write response error".into(),
            AttemptError::Timeout(TransferTimeout::MaxTime(_)) => "timed out".into(),
            AttemptError::Timeout(TransferTimeout::SpeedLimit { .. }) => "too slow".into(),
        }
    }

    fn retry(&self) -> bool {
        match self {
            AttemptError::Request(_) | AttemptError::Timeout(_) => true,
            // an invalid request will fail the same way again, and local disk failures (e.g. out
            // of space) won't be fixed by downloading again
            AttemptError::InvalidRequest(_) | AttemptError::Write(_) => false,
        }
    }
}

fn request_error_reason(error: &reqwest::Error) -> String {
    // checked first because a connect timeout is also a connect error
    if error.is_timeout() {
        "timed out".into()
    } else if error.is_connect() {
        "connection refused".into()
    } else if let Some(status_code) = error.status() {
        status_code
            .canonical_reason()
            .map(ToString::to_string)
            .unwrap_or(format!("status: {status_code}"))
    } else {
        error.to_string()
    }
}

/// Writes the error and its chain of sources, e.g. `error sending request: connection refused`.
impl fmt::Display for AttemptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let error: &dyn std::error::Error = match self {
            AttemptError::InvalidRequest(source) | AttemptError::Request(source) => source,
            AttemptError::Write(source) => source,
            AttemptError::Timeout(source) => source,
        };
        write!(f, "{error}")?;
        let mut source = error.source();
        while let Some(error) = source {
            write!(f, ": {error}")?;
            source = error.source();
        }
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct FailedAttempt {
    pub(crate) error: AttemptError,
    pub(crate) duration: Duration,
}

/// Why [`get`] stopped retrying.
#[derive(Debug, Clone, Copy)]
pub(crate) enum StopReason {
    NotRetryable,
    /// All of the configured `retry` count was used.
    RetriesExhausted,
    /// The next retry wouldn't start within `retry_max_time`.
    RetryMaxTime,
}

impl fmt::Display for StopReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StopReason::NotRetryable => write!(f, "The error can't be retried"),
            StopReason::RetriesExhausted => write!(f, "No retries left"),
            StopReason::RetryMaxTime => write!(f, "Retry time limit exceeded"),
        }
    }
}

#[derive(Debug)]
pub(crate) struct GetError {
    /// Every failed attempt in order, so the last one is the error [`get`] gave up on.
    pub(crate) attempts: Vec<FailedAttempt>,
    pub(crate) stop_reason: StopReason,
    /// The total time spent across all attempts, including retry delays.
    pub(crate) elapsed: Duration,
    /// The most recent response received across all attempts, which may not be from the last
    /// attempt (a retry can fail before any response arrives). It's `None` if no response was
    /// received at all.
    pub(crate) last_response: Option<Box<LastResponse>>,
}

impl GetError {
    /// The error from the last attempt.
    pub(crate) fn error(&self) -> &AttemptError {
        &self
            .attempts
            .last()
            .expect("GetError should have at least one failed attempt")
            .error
    }

    pub(crate) fn last_response(&self) -> Option<&LastResponse> {
        self.last_response.as_deref()
    }
}

impl fmt::Display for GetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let attempts = self.attempts.len();
        writeln!(
            f,
            "Failed after {attempts} {} over {:.1}s. {}.",
            if attempts == 1 { "attempt" } else { "attempts" },
            self.elapsed.as_secs_f32(),
            self.stop_reason
        )?;
        for (index, attempt) in self.attempts.iter().enumerate() {
            writeln!(
                f,
                "- Attempt {} ({:.1}s) - {}",
                index + 1,
                attempt.duration.as_secs_f32(),
                attempt.error
            )?;
        }
        if let Some(last_response) = self.last_response() {
            write!(
                f,
                "\nResponse headers from the most recent response received:\n{last_response}"
            )?;
        }
        Ok(())
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
    #[builder(default = DEFAULT_MAX_TIME)]
    max_time: Duration,
    #[builder(default = DEFAULT_RETRY)]
    retry: usize,
    /// curl `--retry-delay`: a fixed delay between retries instead of exponential backoff.
    retry_delay: Option<Duration>,
    #[builder(default = DEFAULT_RETRY_MAX_TIME)]
    retry_max_time: Duration,
    #[builder(default = DEFAULT_SPEED_LIMIT)]
    speed_limit: u64,
    #[builder(default = DEFAULT_SPEED_TIME)]
    speed_time: Duration,
}

/// The status and headers of the most recent response received by [`get`].
#[derive(Debug, Clone)]
pub(crate) struct LastResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: HeaderMap,
}

impl fmt::Display for LastResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.status)?;
        for (name, value) in &self.headers {
            write!(f, "\n{name}: {}", String::from_utf8_lossy(value.as_bytes()))?;
        }
        Ok(())
    }
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
        .max_time(download_task.max_time)
        .retry(download_task.retry)
        .maybe_retry_delay(download_task.retry_delay)
        .retry_max_time(download_task.retry_max_time)
        .speed_limit(download_task.speed_limit)
        .speed_time(download_task.speed_time)
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
    let mut hasher = IoWrapper(Sha256::new());
    io::copy(&mut response_body, &mut hasher).map_err(|e| create_write_error(download_task, e))?;
    let digest = hasher.0.finalize();
    if checksum == digest.as_slice() {
        Ok(())
    } else {
        Err(DownloadError::ChecksumMismatch {
            url: download_task.source_url.clone(),
            expected_checksum: hex::encode(checksum),
            actual_checksum: hex::encode(digest),
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
    #[builder(default = DEFAULT_MAX_TIME)]
    max_time: Duration,
    #[builder(default = DEFAULT_RETRY)]
    retry: usize,
    /// curl `--retry-delay`: a fixed delay between retries instead of exponential backoff.
    retry_delay: Option<Duration>,
    #[builder(default = DEFAULT_RETRY_MAX_TIME)]
    retry_max_time: Duration,
    #[builder(default = DEFAULT_SPEED_LIMIT)]
    speed_limit: u64,
    #[builder(default = DEFAULT_SPEED_TIME)]
    speed_time: Duration,
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
    use crate::utils::error_handling::test_util::{
        create_download_error, truncated_body_url, unused_local_url,
    };
    use bullet_stream::{global, strip_ansi};
    use flate2::Compression;
    use flate2::write::GzEncoder;
    use regex::Regex;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_download_will_not_retry_with_invalid_request() {
        let dst = tempfile::NamedTempFile::new().unwrap();
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(&DownloadTask::builder("htp://bad.url", dst.path()).build()).unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert!(matches!(source.error(), AttemptError::InvalidRequest(_)));
                    assert_eq!(source.attempts.len(), 1);
                }
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
    fn test_download_decodes_gzip_content_encoding() {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(b"test").unwrap();
        let body = encoder.finish().unwrap();

        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .and(header("accept-encoding", "gzip"))
                    .respond_with(
                        ResponseTemplate::new(200)
                            .insert_header("content-encoding", "gzip")
                            .set_body_raw(body, "application/json"),
                    )
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        global::with_locked_writer(Vec::<u8>::new(), || {
            download(&DownloadTask::builder(server.uri(), dst.path()).build()).unwrap();
        });

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
            download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .retry_delay(TEST_RETRY_DELAY)
                    .build(),
            )
            .unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[
                request_failed_matcher(server.uri(), "Internal Server Error"),
                retry_attempt_success_matcher(1, DEFAULT_RETRY),
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
                    .retry(2)
                    .retry_delay(TEST_RETRY_DELAY)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert!(matches!(source.stop_reason, StopReason::RetriesExhausted));
                    let message = source.to_string();
                    assert!(message.starts_with("Failed after 3 attempts over "));
                    assert!(message.contains(". No retries left.\n"));
                    assert!(message.contains("\n- Attempt 1 ("));
                    assert!(message.contains("\n- Attempt 3 ("));
                    assert!(message.contains("(404 Not Found)"));
                }
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
    fn test_download_failure_includes_most_recent_response() {
        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(404).insert_header("cf-ray", "test-ray"))
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        match create_download_error(&server.uri()) {
            DownloadError::Download { source, .. } => {
                let last_response = source.last_response().unwrap();
                assert_eq!(last_response.status, StatusCode::NOT_FOUND);
                assert_eq!(last_response.headers["cf-ray"], "test-ray");
            }
            e => panic!("Not the expected error: {e:?}"),
        }
    }

    #[test]
    fn test_download_failure_without_response() {
        match create_download_error(&unused_local_url()) {
            DownloadError::Download { source, .. } => {
                assert!(source.last_response().is_none());
            }
            e => panic!("Not the expected error: {e:?}"),
        }
    }

    #[test]
    fn test_download_failure_after_retry_includes_latest_response() {
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();

        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(500).insert_header("cf-ray", "first"))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(502).insert_header("cf-ray", "second"))
                    .up_to_n_times(1)
                    .expect(1)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        global::with_locked_writer(Vec::<u8>::new(), || {
            match download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .retry(1)
                    .retry_delay(TEST_RETRY_DELAY)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    let last_response = source.last_response().unwrap();
                    assert_eq!(last_response.status, StatusCode::BAD_GATEWAY);
                    assert_eq!(last_response.headers["cf-ray"], "second");
                }
                e => panic!("Not the expected error: {e:?}"),
            }
        });
    }

    #[test]
    fn test_download_body_read_failure_includes_most_recent_response() {
        match create_download_error(&truncated_body_url()) {
            DownloadError::Download { source, .. } => {
                assert!(matches!(source.error(), AttemptError::Request(_)));
                assert_eq!(source.last_response().unwrap().status, StatusCode::OK);
            }
            e => panic!("Not the expected error: {e:?}"),
        }
    }

    #[test]
    fn test_download_success_after_retry_of_body_read_failure() {
        let url = raw_http_server(|mut stream, index| {
            if index == 0 {
                // Close the connection before the advertised body length is sent.
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\npartial")
                    .unwrap();
                stream.shutdown(std::net::Shutdown::Write).unwrap();
            } else {
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ntest")
                    .unwrap();
            }
        });
        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            download(
                &DownloadTask::builder(&url, dst.path())
                    .retry_delay(TEST_RETRY_DELAY)
                    .build(),
            )
            .unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[
                request_failed_matcher(&url, "request or response body error"),
                retry_attempt_success_matcher(1, DEFAULT_RETRY),
                saving_matcher(),
            ],
        );
        assert_eq!(fs::read_to_string(dst.path()).unwrap(), "test");
    }

    #[test]
    fn test_last_response_display() {
        let mut headers = HeaderMap::new();
        headers.insert("cf-ray", "test-ray".parse().unwrap());
        headers.insert("server", "cloudflare".parse().unwrap());
        let last_response = LastResponse {
            status: StatusCode::NOT_FOUND,
            headers,
        };
        assert_eq!(
            last_response.to_string(),
            "404 Not Found\ncf-ray: test-ray\nserver: cloudflare"
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
    fn test_download_stops_retrying_when_retry_max_time_is_exceeded() {
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();

        let server = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(async {
                let server = MockServer::start().await;

                Mock::given(method("GET"))
                    .and(path("/"))
                    .respond_with(ResponseTemplate::new(500))
                    .expect(2)
                    .mount(&server)
                    .await;

                server
            });

        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(
                &DownloadTask::builder(server.uri(), dst.path())
                    .retry_delay(Duration::from_millis(300))
                    .retry_max_time(Duration::from_millis(500))
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert_eq!(source.attempts.len(), 2);
                    assert!(matches!(source.stop_reason, StopReason::RetryMaxTime));
                    assert!(source.to_string().contains(". Retry time limit exceeded."));
                }
                e => panic!("Not the expected error: {e:?}"),
            }
        });

        assert_log_contains_matches(
            &log,
            &[
                request_failed_matcher(server.uri(), "Internal Server Error"),
                retry_attempt_failed_matcher(1, 5, "Internal Server Error"),
            ],
        );
    }

    #[test]
    fn test_retry_delays_use_curl_backoff_by_default() {
        let secs = |secs: &[u64]| {
            secs.iter()
                .map(|s| Duration::from_secs(*s))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            retry_delays(&GetRequest::builder("https://test").build()),
            secs(&[1, 2, 4, 8, 16])
        );
        assert_eq!(
            retry_delays(&GetRequest::builder("https://test").retry(12).build()),
            secs(&[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 600, 600])
        );
        assert_eq!(
            retry_delays(
                &GetRequest::builder("https://test")
                    .retry_delay(Duration::from_secs(3))
                    .retry(3)
                    .build()
            ),
            secs(&[3, 3, 3])
        );
    }

    #[test]
    fn test_download_fails_when_max_time_is_exceeded() {
        let url = raw_http_server(|stream, _| trickle_response(stream));
        let dst = tempfile::NamedTempFile::new().unwrap();
        let mut elapsed = Duration::ZERO;

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            // timed inside the lock so waiting on other tests' output isn't counted
            let start = Instant::now();
            match download(
                &DownloadTask::builder(&url, dst.path())
                    .max_time(Duration::from_millis(500))
                    .retry(0)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert!(matches!(
                        source.error(),
                        AttemptError::Timeout(TransferTimeout::MaxTime(_))
                    ));
                }
                e => panic!("Not the expected error: {e:?}"),
            }
            elapsed = start.elapsed();
        });

        assert!(elapsed < Duration::from_secs(2));
        assert_log_contains_matches(&log, &[request_failed_matcher(&url, "timed out")]);
    }

    #[test]
    fn test_download_max_time_resets_on_retry() {
        let url = raw_http_server(|mut stream, index| {
            if index == 0 {
                trickle_response(stream);
            } else {
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\n\r\ntest")
                    .unwrap();
            }
        });
        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            download(
                &DownloadTask::builder(&url, dst.path())
                    .max_time(Duration::from_millis(300))
                    .retry_delay(TEST_RETRY_DELAY)
                    .build(),
            )
            .unwrap();
        });

        assert_log_contains_matches(
            &log,
            &[
                request_failed_matcher(&url, "timed out"),
                retry_attempt_success_matcher(1, DEFAULT_RETRY),
                saving_matcher(),
            ],
        );
        assert_eq!(fs::read_to_string(dst.path()).unwrap(), "test");
    }

    #[test]
    fn test_download_is_not_retried_when_max_time_uses_up_retry_max_time() {
        let url = raw_http_server(|stream, _| trickle_response(stream));
        let dst = tempfile::NamedTempFile::new().unwrap();

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            match download(
                &DownloadTask::builder(&url, dst.path())
                    .max_time(Duration::from_millis(300))
                    .retry_max_time(Duration::from_millis(300))
                    .retry_delay(TEST_RETRY_DELAY)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert_eq!(source.attempts.len(), 1);
                    assert!(matches!(source.stop_reason, StopReason::RetryMaxTime));
                    assert!(matches!(
                        source.error(),
                        AttemptError::Timeout(TransferTimeout::MaxTime(_))
                    ));
                }
                e => panic!("Not the expected error: {e:?}"),
            }
        });

        assert_log_contains_matches(&log, &[request_failed_matcher(&url, "timed out")]);
    }

    // Sends the body one byte every 50ms, so reads never stall but the transfer is very slow.
    fn trickle_response(mut stream: TcpStream) {
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n")
            .unwrap();
        for _ in 0..1000 {
            // stop once the client gives up
            if stream.write_all(b"x").is_err() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn test_download_fails_when_transfer_stalls() {
        // Sends part of the body, then stops sending without closing the connection.
        let url = raw_http_server(|mut stream, _| {
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\npartial")
                .unwrap();
            thread::sleep(Duration::from_secs(5));
        });
        let dst = tempfile::NamedTempFile::new().unwrap();
        let mut elapsed = Duration::ZERO;

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            // timed inside the lock so waiting on other tests' output isn't counted
            let start = Instant::now();
            match download(
                &DownloadTask::builder(&url, dst.path())
                    .speed_time(Duration::from_millis(200))
                    .retry(0)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert!(matches!(
                        source.error(),
                        AttemptError::Timeout(TransferTimeout::SpeedLimit { .. })
                    ));
                }
                e => panic!("Not the expected error: {e:?}"),
            }
            elapsed = start.elapsed();
        });

        assert!(elapsed < Duration::from_secs(2));
        assert_log_contains_matches(&log, &[request_failed_matcher(&url, "too slow")]);
    }

    #[test]
    fn test_download_fails_when_below_speed_limit() {
        let url = raw_http_server(|stream, _| trickle_response(stream));
        let dst = tempfile::NamedTempFile::new().unwrap();
        let mut elapsed = Duration::ZERO;

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            // timed inside the lock so waiting on other tests' output isn't counted
            let start = Instant::now();
            match download(
                &DownloadTask::builder(&url, dst.path())
                    .speed_limit(1000)
                    .speed_time(Duration::from_millis(300))
                    .retry(0)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert!(matches!(
                        source.error(),
                        AttemptError::Timeout(TransferTimeout::SpeedLimit { .. })
                    ));
                    assert!(source.to_string().contains(". No retries left.\n"));
                }
                e => panic!("Not the expected error: {e:?}"),
            }
            elapsed = start.elapsed();
        });

        assert!(elapsed < Duration::from_secs(2));
        assert_log_contains_matches(&log, &[request_failed_matcher(&url, "too slow")]);
    }

    #[test]
    fn test_download_success_when_above_speed_limit() {
        // ~2500 bytes/sec for ~500ms, so the speed is checked many times over several windows
        let url = raw_http_server(|stream, _| paced_response(stream, &[(50, 25, 20)]));
        let dst = tempfile::NamedTempFile::new().unwrap();

        global::with_locked_writer(Vec::<u8>::new(), || {
            download(
                &DownloadTask::builder(&url, dst.path())
                    .speed_limit(100)
                    .speed_time(Duration::from_millis(100))
                    .retry(0)
                    .build(),
            )
            .unwrap();
        });

        assert_eq!(fs::read(dst.path()).unwrap().len(), 50 * 25);
    }

    #[test]
    fn test_download_success_when_briefly_below_speed_limit() {
        // A fast start, a pause shorter than `speed_time`, then fast again. The average over the
        // first `speed_time` is below the limit, but the speed wasn't below it for all of
        // `speed_time`, so curl (and this) keeps going.
        let url = raw_http_server(|mut stream, _| {
            paced_response_head(&mut stream, 20 * 20 + 100 * 20);
            paced_body(&mut stream, &[(20, 20, 10)]);
            thread::sleep(Duration::from_millis(300));
            paced_body(&mut stream, &[(100, 20, 10)]);
        });
        let dst = tempfile::NamedTempFile::new().unwrap();

        global::with_locked_writer(Vec::<u8>::new(), || {
            download(
                &DownloadTask::builder(&url, dst.path())
                    .speed_limit(1000)
                    .speed_time(Duration::from_millis(500))
                    .retry(0)
                    .build(),
            )
            .unwrap();
        });

        assert_eq!(fs::read(dst.path()).unwrap().len(), 20 * 20 + 100 * 20);
    }

    #[test]
    fn test_download_fails_when_response_headers_stall() {
        // Accepts the request but never responds.
        let url = raw_http_server(|_stream, _| thread::sleep(Duration::from_secs(5)));
        let dst = tempfile::NamedTempFile::new().unwrap();
        let mut elapsed = Duration::ZERO;

        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            // timed inside the lock so waiting on other tests' output isn't counted
            let start = Instant::now();
            match download(
                &DownloadTask::builder(&url, dst.path())
                    .speed_time(Duration::from_millis(200))
                    .retry(0)
                    .build(),
            )
            .unwrap_err()
            {
                DownloadError::Download { source, .. } => {
                    assert!(
                        matches!(source.error(), AttemptError::Request(error) if error.is_timeout())
                    );
                    assert!(source.last_response().is_none());
                }
                e => panic!("Not the expected error: {e:?}"),
            }
            elapsed = start.elapsed();
        });

        assert!(elapsed < Duration::from_secs(1));
        assert_log_contains_matches(&log, &[request_failed_matcher(&url, "timed out")]);
    }

    fn paced_response(mut stream: TcpStream, chunks: &[(usize, usize, u64)]) {
        let length = chunks.iter().map(|(size, count, _)| size * count).sum();
        paced_response_head(&mut stream, length);
        paced_body(&mut stream, chunks);
    }

    fn paced_response_head(stream: &mut TcpStream, content_length: usize) {
        stream
            .write_all(
                format!("HTTP/1.1 200 OK\r\nContent-Length: {content_length}\r\n\r\n").as_bytes(),
            )
            .unwrap();
    }

    // Writes `count` chunks of `size` bytes, sleeping `interval_ms` after each.
    fn paced_body(stream: &mut TcpStream, chunks: &[(usize, usize, u64)]) {
        for (size, count, interval_ms) in chunks {
            for _ in 0..*count {
                stream.write_all(&vec![b'x'; *size]).unwrap();
                thread::sleep(Duration::from_millis(*interval_ms));
            }
        }
    }

    // Keeps tests that aren't about the backoff from sleeping through it.
    const TEST_RETRY_DELAY: Duration = Duration::from_millis(10);

    /// A server that runs `handler` with each connection (and its index) after reading the request.
    fn raw_http_server(handler: impl Fn(TcpStream, usize) + Send + Sync + 'static) -> String {
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let handler = std::sync::Arc::new(handler);
        thread::spawn(move || {
            for (index, stream) in listener.incoming().enumerate() {
                let mut stream = stream.unwrap();
                let handler = handler.clone();
                thread::spawn(move || {
                    let mut request = Vec::new();
                    let mut buf = [0; 1024];
                    while !request.ends_with(b"\r\n\r\n") {
                        let n = stream.read(&mut buf).unwrap();
                        if n == 0 {
                            break;
                        }
                        request.extend_from_slice(&buf[..n]);
                    }
                    handler(stream, index);
                });
            }
        });
        url
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
