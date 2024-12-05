// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{
    assert_contains, assert_contains_match, assert_not_contains, BuildpackReference,
    ContainerConfig,
};
use std::time::Duration;
use test_support::{
    assert_web_response, custom_buildpack, integration_test_with_config, nodejs_integration_test,
    nodejs_integration_test_with_config, set_node_engine, wait_for, PORT,
};

const APPLICATION_STARTUP_TIMEOUT: Duration = Duration::from_secs(10);
const METRICS_SEND_INTERVAL: Duration = Duration::from_secs(10);
const METRICS_SEND_TIMEOUT: Duration = Duration::from_secs(15);

#[test]
#[ignore]
fn simple_indexjs() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        assert_contains!(ctx.pack_stdout, "Node.js version not specified, using 22.x");
        assert_contains!(ctx.pack_stdout, "Installing Node.js");
        assert_web_response(&ctx, "node-with-indexjs");
    });
}

#[test]
#[ignore]
fn simple_serverjs() {
    nodejs_integration_test("./fixtures/node-with-serverjs", |ctx| {
        assert_contains!(ctx.pack_stdout, "Detected Node.js version range: 16.0.0");
        if cfg!(target_arch = "aarch64") {
            assert_contains!(
                ctx.pack_stdout,
                "Downloading Node.js 16.0.0 (linux-arm64) from https://nodejs.org/download/release/v16.0.0/node-v16.0.0-linux-arm64.tar.gz"
            );
            assert_contains!(ctx.pack_stdout, "Installing Node.js 16.0.0 (linux-arm64)");
        } else {
            assert_contains!(
                ctx.pack_stdout,
                "Downloading Node.js 16.0.0 (linux-amd64) from https://nodejs.org/download/release/v16.0.0/node-v16.0.0-linux-x64.tar.gz"
            );
            assert_contains!(ctx.pack_stdout, "Installing Node.js 16.0.0 (linux-amd64)");
        }
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
fn heroku_available_parallelism_is_set_at_build_and_runtime() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |_| {},
        |ctx| {
            assert_contains_match!(ctx.pack_stdout, "HEROKU_AVAILABLE_PARALLELISM=\\d+");
            assert_contains_match!(
                ctx.run_shell_command("env").stdout,
                "HEROKU_AVAILABLE_PARALLELISM=\\d+"
            );
        },
        &[
            BuildpackReference::WorkspaceBuildpack(buildpack_id!("heroku/nodejs")),
            BuildpackReference::Other(
                custom_buildpack()
                    .id("test/echo-build-parallelism")
                    .build(indoc! { "
                        #!/usr/bin/env bash
                        env
                    " })
                    .call(),
            ),
        ],
    );
}

#[test]
#[ignore]
fn runtime_metrics_script_is_activated_when_heroku_metrics_url_is_set() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        let mut container_config = ContainerConfig::new();
        let metrics_send_interval = METRICS_SEND_INTERVAL.as_millis().to_string();
        container_config
            .expose_port(PORT)
            .env("NODE_DEBUG", "heroku")
            .env("HEROKU_METRICS_URL", "http://localhost:3000")
            .env("METRICS_INTERVAL_OVERRIDE", &metrics_send_interval);

        ctx.start_container(container_config, |container| {
            wait_for(
                || {
                    assert_contains!(container.logs_now().stdout, "App started");
                },
                APPLICATION_STARTUP_TIMEOUT,
            );

            let stderr = container.logs_now().stderr;
            assert_contains!(stderr, "Registering metrics instrumentation");
            assert_contains!(
                stderr,
                "HEROKU_METRICS_URL set to \"http://localhost:3000\""
            );
            assert_contains!(
                stderr,
                &format!("METRICS_INTERVAL_OVERRIDE set to \"{metrics_send_interval}\"")
            );
            assert_contains!(
                stderr,
                &format!("Using interval of {metrics_send_interval}ms")
            );

            wait_for(
                || {
                    assert_contains!(
                        container.logs_now().stderr,
                        "Sending metrics to http://localhost:3000"
                    );
                },
                METRICS_SEND_TIMEOUT,
            );
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
            wait_for(
                || {
                    assert_contains!(container.logs_now().stdout, "App started");
                },
                APPLICATION_STARTUP_TIMEOUT,
            );
            let stderr = container.logs_now().stderr;
            assert_contains!(stderr, "Registering metrics instrumentation");
            assert_contains!(stderr, "HEROKU_METRICS_URL was not set in the environment");
            assert_contains!(stderr, "Metrics will not be collected for this application");
        });
    });
}

#[test]
#[ignore]
fn runtime_metrics_script_is_activated_when_node_version_is_at_least_v14_10_0() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "14.10.0");
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Downloading Node.js 14.10.0");
            assert_contains!(ctx.pack_stdout, "Installing application metrics scripts");

            let mut container_config = ContainerConfig::new();
            container_config
                .expose_port(PORT)
                .env("NODE_DEBUG", "heroku")
                .env("HEROKU_METRICS_URL", "http://localhost:3000");

            ctx.start_container(container_config, |container| {
                wait_for(
                    || {
                        assert_contains!(container.logs_now().stdout, "App started");
                    },
                    APPLICATION_STARTUP_TIMEOUT,
                );
                assert_contains!(
                    container.logs_now().stderr,
                    "Registering metrics instrumentation"
                );
            });
        },
    );
}

#[test]
#[ignore]
fn runtime_metrics_script_is_not_activated_when_node_version_is_less_than_v14_10_0() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "14.9.0");
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Downloading Node.js 14.9.0");
            assert_contains!(
                ctx.pack_stdout,
                "Not installing application metrics scripts"
            );

            let mut container_config = ContainerConfig::new();
            container_config
                .expose_port(PORT)
                .env("NODE_DEBUG", "heroku");

            ctx.start_container(container_config, |container| {
                wait_for(
                    || {
                        assert_contains!(container.logs_now().stdout, "App started");
                    },
                    APPLICATION_STARTUP_TIMEOUT,
                );
                assert_not_contains!(
                    container.logs_now().stderr,
                    "Registering metrics instrumentation"
                );
            });
        },
    );
}
