#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, assert_not_contains};
use test_support::Builder::{Heroku20, Heroku22};
use test_support::{
    assert_health_check_responds, get_function_invoker_build_config, test_node_function,
};

#[test]
#[ignore]
fn simple_javascript_function_heroku_20() {
    test_node_function("simple-function", Heroku20, |ctx| {
        assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn simple_javascript_function_heroku_22() {
    test_node_function("simple-function", Heroku22, |ctx| {
        assert_health_check_responds(&ctx);
    });
}

#[test]
#[ignore]
fn simple_typescript_function_heroku_20() {
    test_node_function("simple-typescript-function", Heroku20, |ctx| {
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
fn upgrade_simple_nodejs_function_from_heroku20_to_heroku22() {
    test_node_function("simple-function", Heroku20, |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            "Installing Node.js Function Invoker Runtime"
        );
        assert_health_check_responds(&ctx);
        ctx.rebuild(
            get_function_invoker_build_config("simple-function", Heroku22),
            |new_ctx| {
                assert_contains!(
                    new_ctx.pack_stdout,
                    "Installing Node.js Function Invoker Runtime"
                );
                assert_health_check_responds(&new_ctx);
            },
        );
    });
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_js_heroku_20() {
    test_node_function(
        "function-with-explicit-runtime-dependency-js",
        Heroku20,
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_js_heroku_22() {
    test_node_function(
        "function-with-explicit-runtime-dependency-js",
        Heroku22,
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_ts_heroku_20() {
    test_node_function(
        "function-with-explicit-runtime-dependency-ts",
        Heroku20,
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_explicit_runtime_dependency_ts_heroku_22() {
    test_node_function(
        "function-with-explicit-runtime-dependency-ts",
        Heroku22,
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_not_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_js_heroku_20() {
    test_node_function(
        "function-with-implicit-runtime-dependency-js",
        Heroku20,
        |ctx| {
            assert_not_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_js_heroku_22() {
    test_node_function(
        "function-with-implicit-runtime-dependency-js",
        Heroku22,
        |ctx| {
            assert_not_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_ts_heroku_20() {
    test_node_function(
        "function-with-implicit-runtime-dependency-ts",
        Heroku20,
        |ctx| {
            assert_not_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions Runtime for Node.js (@heroku/sf-fx-runtime-nodejs) will not be auto-detected and must be added as a dependency in package.json");
            assert_health_check_responds(&ctx);
        },
    );
}

#[test]
#[ignore]
fn test_function_with_implicit_runtime_dependency_ts_heroku_22() {
    test_node_function(
        "function-with-implicit-runtime-dependency-ts",
        Heroku22,
        |ctx| {
            assert_not_contains!(ctx.pack_stdout, "Runtime declared in package.json");
            assert_contains!(ctx.pack_stderr, "Future versions of the Functions runtime for Node.js will not be auto-detected and must be added as a dependency in package.json.");
            assert_health_check_responds(&ctx);
        },
    );
}
