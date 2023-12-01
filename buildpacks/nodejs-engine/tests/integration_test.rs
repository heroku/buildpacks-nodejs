// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::{assert_contains, ContainerConfig};
use std::time::Duration;
use test_support::{
    assert_web_response, nodejs_integration_test, nodejs_integration_test_with_config,
    set_node_engine, PORT,
};

#[test]
#[ignore]
fn simple_indexjs() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        assert_contains!(ctx.pack_stdout, "Node.js version not specified, using 20.x");
        assert_contains!(ctx.pack_stdout, "Installing Node.js");
        assert_web_response(&ctx, "node-with-indexjs");
    });
}

#[test]
#[ignore]
fn simple_serverjs() {
    nodejs_integration_test("./fixtures/node-with-serverjs", |ctx| {
        assert_contains!(ctx.pack_stdout, "Detected Node.js version range: 16.0.0");
        assert_contains!(ctx.pack_stdout, "Installing Node.js 16.0.0");
        assert_web_response(&ctx, "node-with-serverjs");
    });
}

#[test]
#[ignore]
fn reinstalls_node_if_version_changes() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "^14.0");
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Installing Node.js 14");
            let mut config = ctx.config.clone();
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "^16.0");
            });
            ctx.rebuild(config, |ctx| {
                assert_contains!(ctx.pack_stdout, "Installing Node.js 16");
            });
        },
    );
}

#[test]
#[ignore]
fn runtime_metrics_script_is_activated_when_heroku_metrics_url_is_set() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        let mut container_config = ContainerConfig::new();
        container_config
            .expose_port(PORT)
            .env("NODE_DEBUG", "heroku")
            .env("HEROKU_METRICS_URL", "http://localhost:3000")
            .env("METRICS_INTERVAL_OVERRIDE", "10000");

        ctx.start_container(container_config, |container| {
            std::thread::sleep(Duration::from_secs(11));
            let output = container.logs_now();
            assert_contains!(output.stderr, "Registering metrics instrumentation");
            assert_contains!(
                output.stderr,
                "HEROKU_METRICS_URL set to \"http://localhost:3000\""
            );
            assert_contains!(output.stderr, "METRICS_INTERVAL_OVERRIDE set to \"10000\"");
            assert_contains!(output.stderr, "Using interval of 10000ms");
            assert_contains!(output.stderr, "Sending metrics to http://localhost:3000");
        });
    });
}

#[test]
#[ignore]
fn runtime_metrics_script_is_not_activated_when_heroku_metrics_url_is_not_set() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        let mut container_config = ContainerConfig::new();
        container_config
            .expose_port(PORT)
            .env("NODE_DEBUG", "heroku");

        ctx.start_container(container_config, |container| {
            std::thread::sleep(Duration::from_secs(11));
            let output = container.logs_now();
            assert_contains!(output.stderr, "Registering metrics instrumentation");
            assert_contains!(
                output.stderr,
                "HEROKU_METRICS_URL was not set in the environment"
            );
            assert_contains!(
                output.stderr,
                "Metrics will not be collected for this application"
            );
        });
    });
}
