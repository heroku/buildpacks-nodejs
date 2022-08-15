#![warn(clippy::pedantic)]

use libcnb_test::{
    assert_contains, BuildConfig, BuildpackReference, ContainerConfig, ContainerContext, TestRunner,
};
use std::time::Duration;

const PORT: u16 = 8080;

fn test_node_function(fixture: &str, builder: &str) {
    TestRunner::default().build(
        BuildConfig::new(builder, format!("../../test/fixtures/{fixture}")).buildpacks(vec![
            BuildpackReference::Other(String::from("heroku/nodejs-engine")),
            BuildpackReference::Other(String::from("heroku/nodejs-npm")),
            BuildpackReference::Crate,
        ]),
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Installing Node.js Function Invoker Runtime"
            );
            ctx.start_container(ContainerConfig::new().expose_port(PORT), |container| {
                test_function_response(&container);
            });
        },
    );
}

fn test_function_response(container: &ContainerContext) {
    std::thread::sleep(Duration::from_secs(5));
    let addr = container
        .address_for_port(PORT)
        .expect("couldn't get container address");
    let resp = ureq::post(&format!("http://{addr}"))
        .set("x-health-check", "true")
        .call()
        .expect("request to container failed")
        .into_string()
        .expect("response read error");

    assert_contains!(resp, "OK");
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

#[test]
#[ignore]
fn upgrade_simple_nodejs_function_from_heroku20_to_heroku22() {
    let node_function_buildpacks = vec![
        BuildpackReference::Other(String::from("heroku/nodejs-engine")),
        BuildpackReference::Other(String::from("heroku/nodejs-npm")),
        BuildpackReference::Crate,
    ];
    TestRunner::default().build(
        BuildConfig::new(
            "heroku/buildpacks:20",
            "../../test/fixtures/simple-function",
        )
        .buildpacks(node_function_buildpacks.clone()),
        |initial_ctx| {
            assert_contains!(
                initial_ctx.pack_stdout,
                "Installing Node.js Function Invoker Runtime"
            );
            initial_ctx.rebuild(
                BuildConfig::new("heroku/builder:22", "../../test/fixtures/simple-function")
                    .buildpacks(node_function_buildpacks),
                |upgrade_ctx| {
                    assert_contains!(
                        upgrade_ctx.pack_stdout,
                        "Installing Node.js Function Invoker Runtime"
                    );
                    upgrade_ctx.start_container(
                        ContainerConfig::new().expose_port(PORT),
                        |container| {
                            test_function_response(&container);
                        },
                    );
                },
            );
        },
    );
}
