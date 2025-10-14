// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]
// Required due to: https://github.com/rust-lang/rust-clippy/issues/11119
#![allow(clippy::unwrap_used)]

use libcnb_test::assert_contains;
use std::fs;
use std::path::Path;
use test_support::{
    create_build_snapshot, nodejs_integration_test, nodejs_integration_test_with_config,
    set_package_manager,
};

#[test]
#[ignore = "integration test"]
fn npm_engine_install() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
    });
}

#[test]
#[ignore = "integration test"]
fn npm_package_manager_install() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "npm@10.2.0");
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            let output = ctx.run_shell_command("npm --version");
            assert_contains!(output.stdout, "10.2.0");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_npm_engine_caching() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_version_change_invalidates_npm_engine_cache() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);

        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            update_engine_entry(&app_dir, "npm", "9.6.5");
        });

        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_node_version_change_invalidates_npm_engine_cache() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);

        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            update_engine_entry(&app_dir, "node", "16.20.1");
        });

        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
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
