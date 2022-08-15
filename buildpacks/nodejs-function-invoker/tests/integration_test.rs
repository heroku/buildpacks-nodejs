#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, BuildConfig, BuildpackReference, ContainerConfig, TestRunner};
use std::time::Duration;

fn test_node_function(fixture: &str, builder: &str) {
    TestRunner::default().build(
        BuildConfig::new(builder, format!("../../test/fixtures/{fixture}"))
        .buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/nodejs-engine")),
            BuildpackReference::Other(String::from("heroku/nodejs-npm")),
            BuildpackReference::Crate,
        ]),
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Installing Node.js Function Invoker Runtime");
            let port = 8080;
            ctx.start_container(ContainerConfig::new().expose_port(port), |container| {
                std::thread::sleep(Duration::from_secs(1));
                let addr = container
                    .address_for_port(port)
                    .expect("couldn't get container address");
                let resp = ureq::post(&format!("http://{addr}"))
                    .set("x-health-check", "true")
                    .call()
                    .expect("request to container failed")
                    .into_string()
                    .expect("response read error");

                assert_contains!(resp, "OK");
            });
        },
    );
}


#[test]
#[ignore]
fn simple_javascript_function_heroku_20() {
    test_node_function("simple-function", "heroku/buildpacks:20");
}

#[test]
#[ignore]
fn simple_javascript_function_heroku_22() {
    test_node_function("simple-function", "heroku/builder:22");
}

#[test]
#[ignore]
fn simple_typescript_function_heroku_20() {
    test_node_function("simple-typescript-function", "heroku/buildpacks:20");
}
#[test]
#[ignore]
fn simple_typescript_function_heroku_22() {
    test_node_function("simple-typescript-function", "heroku/builder:22");
}
