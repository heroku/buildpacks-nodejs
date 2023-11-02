#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, PackResult};
use test_support::{
    assert_web_response, nodejs_integration_test, nodejs_integration_test_with_config,
    set_node_engine,
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
fn errors_with_dependencies_and_missing_lockfile() {
    nodejs_integration_test_with_config(
        "./fixtures/dependencies-missing-lockfile",
        |cfg| {
            cfg.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stderr,
                "A lockfile from a supported package manager is required"
            );
            assert_contains!(
                ctx.pack_stderr,
                "The package.json for this project specifies dependencies"
            );
            assert_contains!(
                ctx.pack_stderr,
                "To use npm to install dependencies, run `npm install`."
            );
            assert_contains!(
                ctx.pack_stderr,
                "to use yarn to install dependencies, run `yarn install`."
            );
            assert_contains!(
                ctx.pack_stderr,
                "to use pnpm to install dependencies, run `pnpm install`."
            );
        },
    );
}
