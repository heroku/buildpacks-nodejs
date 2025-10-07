use std::sync::LazyLock;
use tokio::runtime::Runtime;

pub(crate) static ASYNC_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .expect("Should be able to construct the Async Runtime")
});
