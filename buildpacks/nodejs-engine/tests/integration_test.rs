// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{assert_contains_match, BuildpackReference};
use test_support::{
    assert_web_response, create_build_snapshot, custom_buildpack, integration_test_with_config,
    nodejs_integration_test, nodejs_integration_test_with_config, set_node_engine,
};

#[test]
#[ignore = "integration test"]
fn simple_indexjs() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        create_build_snapshot(&ctx.pack_stderr).assert();
        assert_web_response(&ctx, "node-with-indexjs");
    });
}

#[test]
#[ignore = "integration test"]
fn simple_serverjs() {
    nodejs_integration_test("./fixtures/node-with-serverjs", |ctx| {
        create_build_snapshot(&ctx.pack_stderr).assert();
        assert_web_response(&ctx, "node-with-serverjs");
    });
}

#[test]
#[ignore = "integration test"]
fn reinstalls_node_if_version_changes() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "^14.0");
            });
        },
        |ctx| {
            let build_snapshot = create_build_snapshot(&ctx.pack_stderr);

            let mut config = ctx.config.clone();
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "^16.0");
            });

            ctx.rebuild(config, |ctx| {
                build_snapshot.rebuild_output(&ctx.pack_stderr).assert();
            });
        },
    );
}

#[test]
#[ignore = "integration test"]
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
#[ignore = "integration test"]
fn node_24() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-serverjs",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "24.x");
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stderr).assert();
            assert_web_response(&ctx, "node-with-serverjs");
        },
    );
}
