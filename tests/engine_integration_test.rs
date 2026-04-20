// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb::data::buildpack_id;
use libcnb_test::{BuildpackReference, PackResult, assert_contains, assert_contains_match};
use nodejs_data::SUPPORTED_NODEJS_VERSIONS;
use test_support::{
    assert_web_response, create_build_snapshot, integration_test_with_config,
    nodejs_integration_test, nodejs_integration_test_with_config, print_build_env_buildpack,
    set_node_engine,
};

#[test]
#[ignore = "integration test"]
fn simple_indexjs() {
    nodejs_integration_test("./fixtures/node-with-indexjs", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "node-with-indexjs");
    });
}

#[test]
#[ignore = "integration test"]
fn simple_serverjs() {
    nodejs_integration_test("./fixtures/node-with-serverjs", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
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
                set_node_engine(&app_dir, "^20.0");
            });
        },
        |ctx| {
            let build_snapshot = create_build_snapshot(&ctx.pack_stdout);

            let mut config = ctx.config.clone();
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "^22.0");
            });

            ctx.rebuild(config, |ctx| {
                build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            });
        },
    );
}

#[test]
#[ignore = "integration test"]
fn node_build_and_runtime_env() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |_| {},
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();

            let env = ctx.run_shell_command("env").stdout;
            assert_contains!(
                env,
                "PATH=/workspace/node_modules/.bin:/layers/heroku_nodejs/dist/bin"
            );
            assert_contains!(env, "LD_LIBRARY_PATH=/layers/heroku_nodejs/dist/lib");
            assert_contains_match!(env, "HEROKU_AVAILABLE_PARALLELISM=\\d+");
        },
        &[
            BuildpackReference::WorkspaceBuildpack(buildpack_id!("heroku/nodejs")),
            print_build_env_buildpack(),
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
            create_build_snapshot(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "node-with-serverjs");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn node_25() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-serverjs",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_node_engine(&app_dir, "25.x");
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "node-with-serverjs");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn ssl_cert_file_is_respected() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.env("SSL_CERT_FILE", "/dev/null");
            config.env("SSL_CERT_DIR", "/nonexistent");
            config.expected_pack_result(PackResult::Failure);
        },
        |_ctx| {},
    );

    nodejs_integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.env("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
            config.env("SSL_CERT_DIR", "/nonexistent");
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Installing Node.js");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn node_eol_warning() {
    nodejs_integration_test_with_config(
        "./fixtures/node-with-serverjs",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                // Assumes the version one below the lowest supported major is EOL and present in the inventory.
                let unsupported_version = format!("{}.x", SUPPORTED_NODEJS_VERSIONS[0] - 1);
                set_node_engine(&app_dir, &unsupported_version);
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
        },
    );
}
