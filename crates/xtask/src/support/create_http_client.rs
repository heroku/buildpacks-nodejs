use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use std::time::Duration;

pub(crate) fn create_http_client() -> ClientWithMiddleware {
    ClientBuilder::new(
        reqwest::ClientBuilder::new()
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(10))
            .build()
            .expect("Should create a reqwest client"),
    )
    .with(RetryTransientMiddleware::new_with_policy(
        ExponentialBackoff::builder().build_with_max_retries(5),
    ))
    .build()
}
