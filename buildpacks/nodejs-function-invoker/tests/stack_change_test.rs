#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, BuildConfig, BuildpackReference, ContainerConfig, TestRunner};
use std::time::Duration;

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
                    let port = 8080;
                    upgrade_ctx.start_container(
                        ContainerConfig::new().expose_port(port),
                        |container| {
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
                        },
                    );
                },
            );
        },
    );
}
