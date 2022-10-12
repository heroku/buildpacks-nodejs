use libcnb_test::{assert_contains, BuildConfig, BuildpackReference, ContainerConfig, TestContext, TestRunner};
use std::time::Duration;

const PORT: u16 = 8080;

pub enum Builder {
    Heroku20,
    Heroku22
}

pub fn get_function_invoker_buildpacks() -> Vec<BuildpackReference> {
    vec![
        BuildpackReference::Other(String::from("heroku/nodejs-engine")),
        BuildpackReference::Other(String::from("heroku/nodejs-npm")),
        BuildpackReference::Crate,
    ]
}

pub fn get_builder_name(builder: Builder) -> &'static str {
    match builder {
        Builder::Heroku20 => "heroku/buildpacks:20",
        Builder::Heroku22 => "heroku/builder:22"
    }
}

pub fn get_function_invoker_build_config(fixture: &str, builder: Builder) -> BuildConfig {
    BuildConfig::new(
        get_builder_name(builder),
        format!("../../test/fixtures/{fixture}")
    ).buildpacks(
        get_function_invoker_buildpacks()
    ).to_owned()
}

pub fn test_node_function(fixture: &str, builder: Builder, test_body: fn(TestContext)) {
    TestRunner::default().build(
        get_function_invoker_build_config(fixture, builder),
        |ctx| test_body(ctx)
    );
}

pub fn assert_health_check_responds(ctx: &TestContext) {
    ctx.start_container(ContainerConfig::new().expose_port(PORT), |container| {
        std::thread::sleep(Duration::from_secs(5));

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
