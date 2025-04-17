// This module is only used for testing, where using unwrap() is acceptable.
#![allow(clippy::unwrap_used)]

mod snapshot_filters;
mod test_arch;
mod test_builder;

use crate::snapshot_filters::create_snapshot_filters;
use crate::test_arch::{get_test_arch, TestArch};
use crate::test_builder::get_test_builder;
use insta::{assert_snapshot, with_settings};
use libcnb::data::buildpack_id;
use libcnb_test::{
    assert_contains, BuildConfig, BuildpackReference, ContainerConfig, ContainerContext,
    TestContext, TestRunner,
};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};
use std::{fs, panic};

const DEFAULT_BUILDER: &str = "heroku/builder:22";
const PORT: u16 = 8080;
pub const DEFAULT_RETRIES: u32 = 10;
pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(1);

#[must_use]
fn get_integration_test_builder() -> String {
    std::env::var("INTEGRATION_TEST_CNB_BUILDER").unwrap_or(DEFAULT_BUILDER.to_string())
}

pub fn nodejs_integration_test(fixture: &str, test_body: fn(TestContext)) {
    nodejs_integration_test_with_config(fixture, |_| {}, test_body);
}

pub fn nodejs_integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
) {
    integration_test_with_config(
        fixture,
        with_config,
        test_body,
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs"
        ))],
    );
}

pub fn function_integration_test(fixture: &str, test_body: fn(TestContext)) {
    function_integration_test_with_config(fixture, |_| {}, test_body);
}

fn function_integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
) {
    if get_integration_test_builder() != "heroku/builder:22" {
        // only heroku/builder:22 supports functions
        // https://github.com/heroku/cnb-builder-images/blob/main/salesforce-functions/builder.toml
        return;
    }
    integration_test_with_config(
        fixture,
        with_config,
        test_body,
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-function"
        ))],
    );
}

pub fn integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
    buildpacks: &[BuildpackReference],
) {
    let builder = get_test_builder();
    let arch = get_test_arch();
    let app_dir = test_dir().join(fixture);

    let target_triple = match arch {
        TestArch::Arm64 => "aarch64-unknown-linux-musl",
        TestArch::Amd64 => "x86_64-unknown-linux-musl",
    };

    let mut build_config = BuildConfig::new(builder, app_dir);
    build_config.buildpacks(buildpacks);
    build_config.target_triple(target_triple);

    // NOTE: Due to the way that npm & pnpm emit their update notifier message, it
    //       creates non-deterministic output. This creates issues with snapshot testing.
    //       Since this is tool-provided output, I'd like to keep it in customer builds but
    //       prevent from being emitted in test scenarios. By setting the following
    //       environment variable, we can suppress this message in both npm & pnpm.
    build_config.env("npm_config_update_notifier", "false");

    with_config(&mut build_config);

    TestRunner::default().build(build_config, test_body);
}

pub fn retry<T, E>(
    attempts: u32,
    retry_delay: Duration,
    retryable_action: impl Fn() -> Result<T, E>,
) -> Result<T, E> {
    let mut result = retryable_action();
    for _ in 1..attempts {
        if result.is_ok() {
            return result;
        }
        std::thread::sleep(retry_delay);
        result = retryable_action();
    }
    result
}

pub fn start_container(ctx: &TestContext, in_container: impl Fn(&ContainerContext, &SocketAddr)) {
    ctx.start_container(ContainerConfig::new().expose_port(PORT), |container| {
        let socket_addr = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
            std::panic::catch_unwind(|| container.address_for_port(PORT))
        })
        .unwrap();
        in_container(&container, &socket_addr);
    });
}

pub fn assert_web_response(ctx: &TestContext, expected_response_body: &'static str) {
    start_container(ctx, |_container, socket_addr| {
        let mut response = retry(DEFAULT_RETRIES, DEFAULT_RETRY_DELAY, || {
            ureq::get(&format!("http://{socket_addr}/")).call()
        })
        .unwrap();
        let response_body = response.body_mut().read_to_string().unwrap();
        assert_contains!(response_body, expected_response_body);
    });
}

pub fn wait_for<F>(condition: F, max_wait_time: Duration)
where
    F: Fn() + panic::RefUnwindSafe,
{
    let mut error = None;
    let start_time = SystemTime::now();
    while SystemTime::now()
        .duration_since(start_time)
        .expect("should not be an earlier time")
        < max_wait_time
    {
        match panic::catch_unwind(&condition) {
            Ok(()) => return,
            Err(err) => error = Some(err),
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    match error {
        None => panic!("timeout exceeded"),
        Some(error) => panic::resume_unwind(error),
    }
}

pub fn set_node_engine(app_dir: &Path, version_range: &str) {
    update_package_json(app_dir, |package_json| {
        package_json
            .entry("engines")
            .or_insert(serde_json::Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .unwrap()
            .insert(
                "node".to_string(),
                serde_json::Value::String(version_range.to_string()),
            );
    });
}

pub fn set_package_manager(app_dir: &Path, package_manager: &str) {
    update_package_json(app_dir, |package_json| {
        package_json.insert(
            "packageManager".to_string(),
            serde_json::Value::String(package_manager.to_string()),
        );
    });
}

pub fn add_package_json_dependency(app_dir: &Path, package_name: &str, package_version: &str) {
    update_package_json(app_dir, |json| {
        let dependencies = json["dependencies"].as_object_mut().unwrap();
        dependencies.insert(
            package_name.to_string(),
            serde_json::Value::String(package_version.to_string()),
        );
    });
}

pub fn add_build_script(app_dir: &Path, script: &str) {
    update_package_json(app_dir, |json| {
        let scripts = json
            .entry("scripts")
            .or_insert(serde_json::Value::Object(serde_json::Map::new()))
            .as_object_mut()
            .unwrap();
        scripts.insert(
            script.to_string(),
            serde_json::Value::String(format!("echo 'executed {script}'")),
        );
    });
}

fn update_package_json(
    app_dir: &Path,
    update: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>),
) {
    update_json_file(&app_dir.join("package.json"), |json| {
        update(
            json.as_object_mut()
                .expect("Deserialized package.json value should be an object"),
        );
    });
}

pub fn update_json_file(path: &Path, update: impl FnOnce(&mut serde_json::Value)) {
    let json_file = std::fs::read_to_string(path).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&json_file).unwrap();
    update(&mut json);
    let new_contents = serde_json::to_string(&json).unwrap();
    std::fs::write(path, new_contents).unwrap();
}

#[bon::builder(on(String, into))]
pub fn custom_buildpack(id: &str, detect: Option<String>, build: Option<String>) -> String {
    let buildpack_dir = tempfile::tempdir().unwrap().into_path();
    let bin_dir = buildpack_dir.join("bin");

    fs::create_dir(&bin_dir).unwrap();

    fs::write(
        buildpack_dir.join("buildpack.toml"),
        format!(
            "
api = \"0.10\"

[buildpack]
id = \"{id}\"
version = \"0.0.0\"
    "
        ),
    )
    .unwrap();

    fs::write(
        bin_dir.join("detect"),
        detect.unwrap_or("#!/usr/bin/env bash".to_string()),
    )
    .unwrap();

    fs::write(
        bin_dir.join("build"),
        build.unwrap_or("#!/usr/bin/env bash".to_string()),
    )
    .unwrap();

    buildpack_dir.to_string_lossy().to_string()
}

fn workspace_package_dir() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("The CARGO_MANIFEST_DIR should be automatically set by Cargo when running tests but it was not")
}

fn test_dir() -> PathBuf {
    workspace_package_dir().join("tests")
}

fn snapshots_dir() -> PathBuf {
    test_dir().join("snapshots")
}

#[must_use]
pub fn test_name() -> String {
    std::thread::current()
        .name()
        .expect("Test name should be available as the current thread name")
        .to_string()
}

const REBUILD_SEPARATOR: &str = "\
--------------------------------------------- REBUILD ---------------------------------------------";

#[bon::builder(finish_fn = assert)]
#[allow(clippy::needless_pass_by_value)]
pub fn create_build_snapshot(
    #[builder(start_fn, into)] //
    build_output: String,
    #[builder(into)] //
    rebuild_output: Option<String>,
) {
    let filters = create_snapshot_filters();

    let snapshot_output = if let Some(rebuild_output) = rebuild_output {
        format!("{build_output}\n{REBUILD_SEPARATOR}\n{rebuild_output}")
    } else {
        build_output
    };

    with_settings!({
        omit_expression => true,
        prepend_module_to_snapshot => false,
        snapshot_path => snapshots_dir(),
        filters => filters.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }, {
        assert_snapshot!(test_name(), snapshot_output);
    });
}
