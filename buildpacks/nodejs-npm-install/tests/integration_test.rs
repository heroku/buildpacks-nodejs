use indoc::indoc;
use libcnb_test::assert_contains;
use serde_json::json;
use std::fs;
use std::path::Path;
use test_support::nodejs_integration_test;

#[test]
#[ignore = "integration test"]
fn test_npm_install() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Heroku npm Install Buildpack]
                npm version: 6.14.18
                Creating new npm cache
            " }
            .trim()
        );
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Installing dependencies]
                added 4 packages
            " }
            .trim()
        );
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Running scripts]
                No build scripts found
            " }
            .trim()
        )
    })
}

#[test]
#[ignore = "integration test"]
fn test_npm_install_caching() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Heroku npm Install Buildpack]
                npm version: 6.14.18
                Creating new npm cache
            " }
            .trim()
        );
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Installing dependencies]
                added 4 packages
            " }
            .trim()
        );
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    [Heroku npm Install Buildpack]
                    npm version: 6.14.18
                    Restoring npm cache
                " }
                .trim()
            );
        })
    })
}

#[test]
#[ignore = "integration test"]
fn test_npm_install_new_package() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Heroku npm Install Buildpack]
                npm version: 6.14.18
                Creating new npm cache
            " }
            .trim()
        );
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Installing dependencies]
                added 4 packages
            " }
            .trim()
        );

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
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    [Heroku npm Install Buildpack]
                    npm version: 6.14.18
                    Restoring npm cache
                " }
                .trim()
            );
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    [Installing dependencies]
                    added 5 packages
                " }
                .trim()
            );
        })
    })
}

fn add_package_json_dependency(app_dir: &Path, package_name: &str, package_version: &str) {
    let package_json = fs::read_to_string(app_dir.join("package.json")).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&package_json).unwrap();
    let dependencies = json["dependencies"].as_object_mut().unwrap();
    dependencies.insert(
        package_name.to_string(),
        serde_json::Value::String(package_version.to_string()),
    );
    let new_package_json = serde_json::to_string(&json).unwrap();
    fs::write(app_dir.join("package.json"), new_package_json).unwrap();
}

fn add_lockfile_entry(app_dir: &Path, package_name: &str, lockfile_entry: serde_json::Value) {
    let package_lock_json = fs::read_to_string(app_dir.join("package-lock.json")).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&package_lock_json).unwrap();
    let dependencies = json["dependencies"].as_object_mut().unwrap();
    dependencies.insert(package_name.to_string(), lockfile_entry);
    let new_package_lock_json = serde_json::to_string(&json).unwrap();
    fs::write(app_dir.join("package-lock.json"), new_package_lock_json).unwrap();
}
