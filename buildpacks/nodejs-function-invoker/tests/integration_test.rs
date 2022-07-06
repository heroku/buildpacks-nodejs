#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, BuildpackReference, TestConfig, TestRunner};
use std::time::Duration;

#[test]
#[ignore]
fn nodejs_function_invoker_simple_function() {
    TestRunner::default().run_test(
        TestConfig::new(
            "heroku/buildpacks:20",
            "../../test/fixtures/simple-function",
        )
        .buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/nodejs-engine")),
            BuildpackReference::Crate,
        ]),
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Installing Node.js Function Invoker Runtime"
            );
            let port = 8080;
            ctx.prepare_container()
                .expose_port(port)
                .start_with_default_process(|container| {
                    std::thread::sleep(Duration::from_secs(5));
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
fn nodejs_function_invoker_simple_typescript_function() {
    TestRunner::default().run_test(
        TestConfig::new(
            "heroku/buildpacks:20",
            "../../test/fixtures/simple-typescript-function",
        )
        .buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/nodejs-engine")),
            BuildpackReference::Other(String::from("heroku/nodejs-npm")),
            BuildpackReference::Crate,
        ]),
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Installing Node.js Function Invoker Runtime"
            );
            let port = 8080;
            ctx.prepare_container()
                .expose_port(port)
                .start_with_default_process(|container| {
                    std::thread::sleep(Duration::from_secs(5));
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
