use libcnb::data::buildpack_id;
use libcnb_test::{
    assert_contains, BuildConfig, BuildpackReference, ContainerConfig, ContainerContext,
    TestContext, TestRunner,
};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, fs, panic};

pub const PORT: u16 = 8080;
pub const DEFAULT_RETRIES: u32 = 10;
pub const DEFAULT_RETRY_DELAY_IN_SECONDS: u64 = 1;

const DEFAULT_BUILDER: &str = "heroku/builder:22";

pub fn get_integration_test_builder() -> String {
    env::var("INTEGRATION_TEST_CNB_BUILDER").unwrap_or(DEFAULT_BUILDER.to_string())
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

pub fn function_integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
) {
    if get_integration_test_builder() == "heroku/builder:20" {
        // builder:20 doesn't ship with functions
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

fn integration_test_with_config(
    fixture: &str,
    with_config: fn(&mut BuildConfig),
    test_body: fn(TestContext),
    buildpacks: &[BuildpackReference],
) {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("The CARGO_MANIFEST_DIR should be automatically set by Cargo when running tests but it was not");

    let builder = get_integration_test_builder();
    let app_dir = cargo_manifest_dir.join("tests").join(fixture);

    let mut build_config = BuildConfig::new(builder, app_dir);
    build_config.buildpacks(buildpacks);
    with_config(&mut build_config);

    TestRunner::default().build(build_config, test_body);
}

pub fn retry<T, E>(
    attempts: u32,
    retry_delay_in_secs: u64,
    retryable_action: impl Fn() -> Result<T, E>,
    on_success: impl FnOnce(T),
    on_fail: impl FnOnce(E),
) {
    for attempt in 0..attempts {
        match retryable_action() {
            Ok(result) => {
                on_success(result);
                break;
            }
            Err(error) => {
                if attempt == attempts - 1 {
                    on_fail(error);
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_secs(retry_delay_in_secs));
    }
}

pub fn start_container(ctx: &TestContext, in_container: impl Fn(&ContainerContext, &SocketAddr)) {
    ctx.start_container(ContainerConfig::new().expose_port(PORT), |container| {
        retry(
            DEFAULT_RETRIES,
            DEFAULT_RETRY_DELAY_IN_SECONDS,
            || panic::catch_unwind(|| container.address_for_port(PORT)),
            |socket_addr| in_container(&container, &socket_addr),
            |error| panic::resume_unwind(error),
        );
    });
}

pub fn assert_web_response(ctx: &TestContext, expected_response_body: &'static str) {
    start_container(ctx, |_container, socket_addr| {
        retry(
            DEFAULT_RETRIES,
            DEFAULT_RETRY_DELAY_IN_SECONDS,
            || ureq::get(&format!("http://{socket_addr}/")).call(),
            |res| {
                let response_body = res.into_string().unwrap();
                assert_contains!(response_body, expected_response_body)
            },
            |error| panic!("request to assert web response failed: {error}"),
        );
    });
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

pub fn update_package_json(
    app_dir: &Path,
    update: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>),
) {
    update_json_file(&app_dir.join("package.json"), |json| {
        update(
            json.as_object_mut()
                .expect("Deserialized package.json value should be an object"),
        )
    });
}

pub fn update_json_file(path: &Path, update: impl FnOnce(&mut serde_json::Value)) {
    let json_file = fs::read_to_string(path).unwrap();
    let mut json: serde_json::Value = serde_json::from_str(&json_file).unwrap();
    update(&mut json);
    let new_contents = serde_json::to_string(&json).unwrap();
    fs::write(path, new_contents).unwrap();
}
