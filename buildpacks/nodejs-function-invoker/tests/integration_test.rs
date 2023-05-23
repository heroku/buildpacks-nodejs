#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, assert_not_contains};
use test_support::Builder::Heroku22;
use test_support::{assert_health_check_responds, test_node_function};

#[test]
#[ignore]
fn simple_javascript_function_heroku_22() {
    test_node_function("simple-function", Heroku22, |ctx| {
        assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn simple_typescript_function_heroku_22() {
    test_node_function("simple-typescript-function", Heroku22, |ctx| {
        assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_js_heroku_22() {
    test_node_function(
        "functions/with-explicit-runtime-dependency-js",
        Heroku22,
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_ts_heroku_22() {
    test_node_function(
        "functions/with-explicit-runtime-dependency-ts",
        Heroku22,
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_js_heroku_22() {
    test_node_function(
        "functions/with-implicit-runtime-dependency-js",
        Heroku22,
        |ctx| {
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_not_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_ts_heroku_22() {
    test_node_function(
        "functions/with-implicit-runtime-dependency-ts",
        Heroku22,
        |ctx| {
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_not_contains!(
                ctx.pack_stdout,
                "Node.js function runtime declared in package.json"
            );
            assert_health_check_responds(&ctx);
        },
    );
}
