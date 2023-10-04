use heroku_nodejs_utils::package_manager::PackageManager;
use libcnb_test::{assert_contains, assert_not_contains};
use serde_json::json;
use std::path::Path;
use test_support::{
    add_build_script, add_package_json_dependency, nodejs_integration_test,
    nodejs_integration_test_with_config, update_json_file,
};

#[test]
#[ignore = "integration test"]
fn test_npm_install_with_lockfile() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        assert_contains!(ctx.pack_stdout, "# Heroku npm Engine Buildpack");
        assert_contains!(ctx.pack_stdout, "- Installing node modules");
        assert_contains!(ctx.pack_stdout, "- Using npm version `6.14.18`");
        assert_contains!(ctx.pack_stdout, "- Creating npm cache");
        assert_contains!(ctx.pack_stdout, "- Configuring npm cache directory");
        assert_contains!(ctx.pack_stdout, "- Running `npm ci \"--production=false\"`");
        assert_contains!(ctx.pack_stdout, "added 4 packages");
        assert_contains!(ctx.pack_stdout, "- Running scripts");
        assert_contains!(ctx.pack_stdout, "- No build scripts found");
        assert_contains!(ctx.pack_stdout, "- Configuring default processes");
        assert_contains!(
            ctx.pack_stdout,
            "- Skipping default web process (no start script defined)"
        );
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_install_with_no_lockfile() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                std::fs::remove_file(app_dir.join(PackageManager::Npm.lockfile())).unwrap();
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "# Heroku npm Engine Buildpack");
            assert_contains!(ctx.pack_stdout, "- Installing node modules");
            assert_contains!(ctx.pack_stdout, "- Using npm version `6.14.18`");
            assert_contains!(ctx.pack_stdout, "- Creating npm cache");
            assert_contains!(ctx.pack_stdout, "- Configuring npm cache directory");
            assert_contains!(
                ctx.pack_stdout,
                "- Running `npm install --no-package-lock \"--production=false\"`"
            );
            assert_contains!(ctx.pack_stdout, "added 4 packages");
            assert_contains!(ctx.pack_stdout, "- Running scripts");
            assert_contains!(ctx.pack_stdout, "- No build scripts found");
            assert_contains!(ctx.pack_stdout, "- Configuring default processes");
            assert_contains!(
                ctx.pack_stdout,
                "- Skipping default web process (no start script defined)"
            );
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_npm_install_caching() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        assert_contains!(ctx.pack_stdout, "- Creating npm cache");
        assert_contains!(ctx.pack_stdout, "added 4 packages");
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            assert_contains!(ctx.pack_stdout, "- Restoring npm cache");
            assert_contains!(ctx.pack_stdout, "added 4 packages");
        })
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_install_new_package() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        assert_contains!(ctx.pack_stdout, "- Creating npm cache");
        assert_contains!(ctx.pack_stdout, "added 4 packages");

        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            add_package_json_dependency(&app_dir, "dotenv", "16.3.1");
            add_lockfile_entry(
                &app_dir,
                "dotenv",
                json!({
                    "version": "16.3.1",
                    "resolved": "https://registry.npmjs.org/dotenv/-/dotenv-16.3.1.tgz",
                    "integrity": "sha512-IPzF4w4/Rd94bA9imS68tZBaYyBWSCE47V1RGuMrB94iyTOIEwRmVL2x/4An+6mETpLrKJ5hQkB8W4kFAadeIQ=="
                })
            );
        });

        ctx.rebuild(config, |ctx| {
            assert_contains!(ctx.pack_stdout, "- Restoring npm cache");
            assert_contains!(ctx.pack_stdout, "added 5 packages");
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_build_scripts() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                add_build_script(&app_dir, "heroku-prebuild");
                add_build_script(&app_dir, "build");
                add_build_script(&app_dir, "heroku-postbuild");
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "- Running `npm run -s heroku-prebuild`");
            assert_contains!(ctx.pack_stdout, "executed heroku-prebuild");

            assert_contains!(ctx.pack_stdout, "- Running `npm run -s build`");
            assert_contains!(ctx.pack_stdout, "executed build");

            assert_contains!(ctx.pack_stdout, "- Running `npm run -s heroku-postbuild`");
            assert_contains!(ctx.pack_stdout, "executed heroku-postbuild");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_npm_build_scripts_prefers_heroku_build_over_build() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                add_build_script(&app_dir, "heroku-build");
                add_build_script(&app_dir, "build");
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "- Running `npm run -s heroku-build`");
            assert_contains!(ctx.pack_stdout, "executed heroku-build");

            assert_not_contains!(ctx.pack_stdout, "- Running `npm run -s build`");
            assert_not_contains!(ctx.pack_stdout, "executed build");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_npm_start_script_creates_a_web_process_launcher() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                add_build_script(&app_dir, "start");
            });
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "- Adding default web process for `npm start`"
            );
        },
    );
}

fn add_lockfile_entry(app_dir: &Path, package_name: &str, lockfile_entry: serde_json::Value) {
    update_json_file(&app_dir.join("package-lock.json"), |json| {
        let dependencies = json["dependencies"].as_object_mut().unwrap();
        dependencies.insert(package_name.to_string(), lockfile_entry);
    });
}
