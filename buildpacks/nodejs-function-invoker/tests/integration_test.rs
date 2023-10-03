#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, assert_not_contains, TestContext};
use std::net::SocketAddr;
use test_support::{
    function_integration_test, retry, start_container, DEFAULT_RETRIES,
    DEFAULT_RETRY_DELAY_IN_SECONDS,
};

#[test]
#[ignore]
fn simple_javascript_function() {
    function_integration_test("../../../test/fixtures/simple-function", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing Node.js Function Invoker");
        start_container_and_assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn simple_typescript_function() {
    function_integration_test("../../../test/fixtures/simple-typescript-function", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing Node.js Function Invoker");
        start_container_and_assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_js() {
    function_integration_test(
        "../../../test/fixtures/functions/with-explicit-runtime-dependency-js",
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            start_container_and_assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_ts() {
    function_integration_test(
        "../../../test/fixtures/functions/with-explicit-runtime-dependency-ts",
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            start_container_and_assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_js() {
    function_integration_test(
        "../../../test/fixtures/functions/with-implicit-runtime-dependency-js",
        |ctx| {
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_not_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            start_container_and_assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_ts() {
    function_integration_test(
        "../../../test/fixtures/functions/with-implicit-runtime-dependency-ts",
        |ctx| {
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_not_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            start_container_and_assert_health_check_responds(&ctx);
        },
    );
}

fn assert_health_check_responds(socket_addr: &SocketAddr) {
    retry(
        DEFAULT_RETRIES,
        DEFAULT_RETRY_DELAY_IN_SECONDS,
        || {
            ureq::post(&format!("http://{socket_addr}"))
                .set("x-health-check", "true")
                .call()
        },
        |res| {
            let response_body = res.into_string().unwrap();
            assert_contains!(response_body, "OK");
        },
        |error| panic!("request to assert function health check response failed: {error}"),
    );
}

fn start_container_and_assert_health_check_responds(ctx: &TestContext) {
    start_container(ctx, |_container, socket_addr| {
        assert_health_check_responds(socket_addr);
    });
}
