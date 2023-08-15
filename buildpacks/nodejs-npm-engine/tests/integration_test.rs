use indoc::indoc;
use libcnb_test::assert_contains;
use std::fs;
use std::path::Path;
use test_support::nodejs_integration_test;

#[test]
#[ignore = "integration test"]
fn npm_engine_install() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Heroku npm Engine Buildpack]
                Resolved npm version: 9.6.6
                Downloading and extracting npm...
                Installed npm version: 9.6.6
            " }
            .trim()
        );
    })
}

#[test]
#[ignore = "integration test"]
fn test_npm_engine_caching() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Heroku npm Engine Buildpack]
                Resolved npm version: 9.6.6
                Downloading and extracting npm...
                Installed npm version: 9.6.6
            " }
            .trim()
        );
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                [Heroku npm Engine Buildpack]
                Resolved npm version: 9.6.6
                Reusing cached npm...
                Installed npm version: 9.6.6
            " }
                .trim()
            );
        })
    })
}

#[test]
#[ignore = "integration test"]
fn test_npm_version_change_invalidates_npm_engine_cache() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Heroku npm Engine Buildpack]
                Resolved npm version: 9.6.6
                Downloading and extracting npm...
                Installed npm version: 9.6.6
            " }
            .trim()
        );
        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            update_engine_entry(&app_dir, "npm", "9.6.5");
        });
        ctx.rebuild(config, |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                [Heroku npm Engine Buildpack]
                Resolved npm version: 9.6.5
                Downloading and extracting npm...
                Installed npm version: 9.6.5
            " }
                .trim()
            );
        })
    })
}

#[test]
#[ignore = "integration test"]
fn test_node_version_change_invalidates_npm_engine_cache() {
    nodejs_integration_test("./fixtures/npm-engine-project", |ctx| {
        assert_contains!(
            ctx.pack_stdout,
            indoc! { "
                [Heroku npm Engine Buildpack]
                Resolved npm version: 9.6.6
                Downloading and extracting npm...
                Installed npm version: 9.6.6
            " }
            .trim()
        );
        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            update_engine_entry(&app_dir, "node", "16.20.1");
        });
        ctx.rebuild(config, |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                [Heroku npm Engine Buildpack]
                Resolved npm version: 9.6.6
                Downloading and extracting npm...
                Installed npm version: 9.6.6
            " }
                .trim()
            );
        })
    })
}

fn update_engine_entry(app_dir: &Path, engine_key: &str, value: &str) {
    let package_json = fs::read_to_string(app_dir.join("package.json")).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&package_json).unwrap();
    let engines = json["engines"].as_object_mut().unwrap();
    engines[engine_key] = serde_json::Value::String(value.to_string());
    let new_package_json = serde_json::to_string(&json).unwrap();
    fs::write(app_dir.join("package.json"), new_package_json).unwrap();
}
