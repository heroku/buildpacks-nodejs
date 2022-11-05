use libcnb_test::{
    assert_contains, BuildConfig, BuildpackReference, ContainerConfig, TestContext, TestRunner,
};
use std::fmt;
use std::fmt::Formatter;
use std::time::Duration;

const PORT: u16 = 8080;
const TIMEOUT: u64 = 10;

pub enum Builder {
    Heroku20,
    Heroku22,
}

impl fmt::Display for Builder {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            Builder::Heroku20 => write!(f, "heroku/buildpacks:20"),
            Builder::Heroku22 => write!(f, "heroku/builder:22"),
        }
    }
}

pub fn get_function_invoker_buildpacks() -> Vec<BuildpackReference> {
    vec![
        BuildpackReference::Other(String::from("heroku/nodejs-engine")),
        BuildpackReference::Other(String::from("heroku/nodejs-npm")),
        BuildpackReference::Crate,
    ]
}

pub fn get_yarn_buildpacks() -> Vec<BuildpackReference> {
    vec![
        BuildpackReference::Other(String::from("heroku/nodejs-engine")),
        BuildpackReference::Crate,
    ]
}

pub fn get_function_invoker_build_config(fixture: &str, builder: Builder) -> BuildConfig {
    BuildConfig::new(
        builder.to_string(),
        format!("../../test/fixtures/{fixture}"),
    )
    .buildpacks(get_function_invoker_buildpacks())
    .to_owned()
}

pub fn get_yarn_build_config(fixture: &str, builder: Builder) -> BuildConfig {
    BuildConfig::new(
        builder.to_string(),
        format!("../../test/fixtures/{fixture}"),
    )
    .buildpacks(get_yarn_buildpacks())
    .to_owned()
}

pub fn test_node_function(fixture: &str, builder: Builder, test_body: fn(TestContext)) {
    TestRunner::default().build(get_function_invoker_build_config(fixture, builder), |ctx| {
        test_body(ctx)
    });
}

pub fn test_yarn_app(fixture: &str, builder: Builder, test_body: fn(TestContext)) {
    TestRunner::default().build(get_yarn_build_config(fixture, builder), |ctx| {
        test_body(ctx)
    });
}

pub fn assert_health_check_responds(ctx: &TestContext) {
    ctx.start_container(ContainerConfig::new().expose_port(PORT), |container| {
        std::thread::sleep(Duration::from_secs(TIMEOUT));

        let addr = container
            .address_for_port(PORT)
            .expect("couldn't get container address");

        let resp = ureq::post(&format!("http://{0}", addr))
            .set("x-health-check", "true")
            .call()
            .expect("request to container failed")
            .into_string()
            .expect("response read error");

        assert_contains!(resp, "OK")
    })
}

pub fn assert_web_response(ctx: &TestContext, text: &'static str) {
    ctx.start_container(ContainerConfig::new().expose_port(PORT), |container| {
        std::thread::sleep(Duration::from_secs(5));

        let addr = container
            .address_for_port(PORT)
            .expect("couldn't get container address");

        let resp = ureq::get(&format!("http://{}/", addr))
            .call()
            .expect("request to container failed")
            .into_string()
            .expect("response read error");

        assert_contains!(resp, text);
    })
}
