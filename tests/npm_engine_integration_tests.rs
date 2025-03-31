// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]
// Required due to: https://github.com/rust-lang/rust-clippy/issues/11119
#![allow(clippy::unwrap_used)]

use libcnb_test::assert_contains;
use std::fs;
use std::path::Path;
use test_support::nodejs_integration_test;

#[test]
#[ignore = "integration test"]
fn npm_engine_install() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(
            ctx.pack_stderr,
            "Found `engines.npm` version `9.6.6` declared in `package.json`"
        );
        assert_contains!(ctx.pack_stderr, "Resolved version `9.6.6` to `9.6.6`");
        assert_contains!(ctx.pack_stderr, "Downloading");
        assert_contains!(ctx.pack_stderr, "Removing npm bundled with Node.js");
        assert_contains!(ctx.pack_stderr, "Installing requested npm");
        assert_contains!(ctx.pack_stderr, "Successfully installed `npm@9.6.6`");
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_engine_caching() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(ctx.pack_stderr, "Downloading");
        assert_contains!(ctx.pack_stderr, "Successfully installed `npm@9.6.6`");
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            assert_contains!(ctx.pack_stderr, "Using cached npm");
            assert_contains!(ctx.pack_stderr, "Successfully installed `npm@9.6.6`");
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_version_change_invalidates_npm_engine_cache() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(ctx.pack_stderr, "Downloading");
        assert_contains!(ctx.pack_stderr, "Successfully installed `npm@9.6.6`");
        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            update_engine_entry(&app_dir, "npm", "9.6.5");
        });
        ctx.rebuild(config, |ctx| {
            assert_contains!(
                ctx.pack_stderr,
                "Invalidating cached npm (npm version changed)"
            );
            assert_contains!(ctx.pack_stderr, "Downloading");
            assert_contains!(ctx.pack_stderr, "Successfully installed `npm@9.6.5`");
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_node_version_change_invalidates_npm_engine_cache() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(ctx.pack_stderr, "Downloading");
        assert_contains!(ctx.pack_stderr, "Successfully installed `npm@9.6.6`");
        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            update_engine_entry(&app_dir, "node", "16.20.1");
        });
        ctx.rebuild(config, |ctx| {
            assert_contains!(
                ctx.pack_stderr,
                "Invalidating cached npm (node version changed)"
            );
            assert_contains!(ctx.pack_stderr, "Downloading");
            assert_contains!(ctx.pack_stderr, "Successfully installed `npm@9.6.6`");
        });
    });
}

fn update_engine_entry(app_dir: &Path, engine_key: &str, value: &str) {
    let package_json = fs::read_to_string(app_dir.join("package.json")).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&package_json).unwrap();
    let engines = json["engines"].as_object_mut().unwrap();
    engines[engine_key] = serde_json::Value::String(value.to_string());
    let new_package_json = serde_json::to_string(&json).unwrap();
    fs::write(app_dir.join("package.json"), new_package_json).unwrap();
}
