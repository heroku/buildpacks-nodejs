// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]
// Required due to: https://github.com/rust-lang/rust-clippy/issues/11119
#![allow(clippy::unwrap_used)]

use base64::Engine;
use libcnb_test::{assert_contains, assert_not_contains, TestContext};
use rand::RngCore;
use std::net::SocketAddr;
use std::time::Duration;
use test_support::{
    function_integration_test, retry, start_container, wait_for, DEFAULT_RETRIES,
    DEFAULT_RETRY_DELAY,
};

const FUNCTION_LOGGING_TIMEOUT: Duration = Duration::from_secs(5);

#[test]
#[ignore]
fn simple_javascript_function() {
    function_integration_test("./fixtures/simple-function", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing Node.js Function Invoker");
        start_container(&ctx, |container, socket_addr| {
            assert_health_check_responds(socket_addr);
            let payload = serde_json::json!({});
            let result = invoke_function(socket_addr, &payload);
            assert_eq!(result, serde_json::Value::String("hello world".to_string()));
            wait_for(
                || {
                    let container_logs = container.logs_now();
                    assert_contains!(container_logs.stdout, "logging info is a fun 1");
                },
                FUNCTION_LOGGING_TIMEOUT,
            );
        });
    });
}

#[test]
#[ignore]
fn simple_typescript_function() {
    function_integration_test("./fixtures/simple-typescript-function", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing Node.js Function Invoker");
        start_container(&ctx, |_container, socket_addr| {
            assert_health_check_responds(socket_addr);
            let payload = serde_json::json!({});
            let result = invoke_function(socket_addr, &payload);
            assert_eq!(
                result,
                serde_json::Value::String("hello world from typescript".to_string())
            );
        });
    });
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_js() {
    function_integration_test("./fixtures/with-explicit-runtime-dependency-js", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            "Node.js function runtime declared in package.json"
        );
        assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
        start_container_and_assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_ts() {
    function_integration_test("./fixtures/with-explicit-runtime-dependency-ts", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            "Node.js function runtime declared in package.json"
        );
        assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
        start_container_and_assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_js() {
    function_integration_test("./fixtures/with-implicit-runtime-dependency-js", |ctx| {
        assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
        assert_not_contains!(
            ctx.pack_stdout,
            "Node.js function runtime declared in package.json"
        );
        start_container_and_assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_ts() {
    function_integration_test("./fixtures/with-implicit-runtime-dependency-ts", |ctx| {
        assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
        assert_not_contains!(
            ctx.pack_stdout,
            "Node.js function runtime declared in package.json"
        );
        start_container_and_assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn function_with_no_lockfile() {
    function_integration_test("./fixtures/function-with-no-lockfile", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing Node.js Function Invoker");
        assert_contains!(ctx.pack_stdout, "Running npm install without a package-lock.json is deprecated. Check in your lockfile to git.");
        start_container(&ctx, |container, socket_addr| {
            assert_health_check_responds(socket_addr);
            let payload = serde_json::json!({});
            let result = invoke_function(socket_addr, &payload);
            assert_eq!(result, serde_json::Value::String("hello world".to_string()));
            wait_for(
                || {
                    let container_logs = container.logs_now();
                    assert_contains!(container_logs.stdout, "logging info is a fun 1");
                },
                FUNCTION_LOGGING_TIMEOUT,
            );
        });
    });
}

fn invoke_function(socket_addr: &SocketAddr, payload: &serde_json::Value) -> serde_json::Value {
    let id = format!("MyFunction-{}", random_hex_string(10));

    let sf_context = base64_encode_json(&serde_json::json!({
        "apiVersion": "",
        "payloadVersion": "",
        "userContext": {
           "orgId": "",
           "userId": "",
           "username": "",
           "orgDomainUrl": "",
           "onBehalfOfUserId": serde_json::Value::Null,
           "salesforceBaseUrl": ""
         }
    }));

    let ssfn_context = base64_encode_json(&serde_json::json!({
        "resource": "",
        "requestId": "",
        "accessToken": "",
        "apexClassId": serde_json::Value::Null,
        "apexClassFQN": serde_json::Value::Null,
        "functionName": "",
        "functionInvocationId": serde_json::Value::Null
    }));

    let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
        ureq::post(&format!("http://{socket_addr}"))
            .set("Content-Type", "application/json")
            .set("Authorization", "")
            .set("ce-id", &id)
            .set("ce-time", "2020-09-03T20:56:28.297915Z")
            .set("ce-type", "")
            .set("ce-source", "")
            .set("ce-specversion", "1.0")
            .set("ce-sfcontext", &sf_context)
            .set("ce-sffncontext", &ssfn_context)
            .send_json(payload.clone())
    })
        .unwrap();

    response.into_json().expect("expected response to be json")
}

fn assert_health_check_responds(socket_addr: &SocketAddr) {
    let response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
        ureq::post(&format!("http://{socket_addr}"))
            .set("x-health-check", "true")
            .call()
    })
        .unwrap();
    let response_body = response.into_string().unwrap();
    assert_contains!(response_body, "OK");
}

fn start_container_and_assert_health_check_responds(ctx: &TestContext) {
    start_container(ctx, |_container, socket_addr| {
        assert_health_check_responds(socket_addr);
    });
}

fn random_hex_string(length: usize) -> String {
    let mut bytes = Vec::with_capacity(length);
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(&bytes)
}

fn base64_encode_json(value: &serde_json::Value) -> String {
    let json_string = serde_json::to_string(value).expect("Value should be encodable as JSON");
    base64::engine::general_purpose::STANDARD.encode(json_string)
}
