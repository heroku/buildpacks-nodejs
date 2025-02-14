// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::assert_contains;
use test_support::{
    assert_web_response, nodejs_integration_test, nodejs_integration_test_with_config,
    set_node_engine,
};

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
