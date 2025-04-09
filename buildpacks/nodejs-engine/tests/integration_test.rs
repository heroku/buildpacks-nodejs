// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use test_support::{
    assert_web_response, create_build_snapshot, nodejs_integration_test,
    nodejs_integration_test_with_config, set_node_engine,
};

#[test]
#[ignore]
fn simple_indexjs() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        create_build_snapshot(&ctx.pack_stderr).assert();
        assert_web_response(&ctx, "node-with-indexjs");
    });
}

#[test]
#[ignore]
fn simple_serverjs() {
    nodejs_integration_test("./fixtures/node-with-serverjs", |ctx| {
        create_build_snapshot(&ctx.pack_stderr).assert();
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
