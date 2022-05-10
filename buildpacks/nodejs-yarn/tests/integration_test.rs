#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, BuildpackReference, IntegrationTest};
use std::time::Duration;

fn test_yarn(fixture: &str, expected_output: Vec<&str>) {
    for stack in ["heroku/buildpacks:20", "heroku/buildpacks:22"] {
        IntegrationTest::new(stack, format!("../../test/fixtures/{fixture}"))
            .buildpacks(vec![
                BuildpackReference::Other(String::from("heroku/nodejs-engine")),
                BuildpackReference::Crate,
            ])
            .run_test(|ctx| {
                for phrase in expected_output.clone() {
                    assert_contains!(ctx.pack_stdout, phrase);
                }
                let port = 8080;
                ctx.prepare_container()
                    .expose_port(port)
                    .start_with_default_process(|container| {
                        std::thread::sleep(Duration::from_secs(5));
                        let addr = container
                            .address_for_port(port)
                            .expect("couldn't get container address");
                        let resp = ureq::get(&format!("http://{addr}"))
                            .call()
                            .expect("request to container failed")
                            .into_string()
                            .expect("response read error");

                        assert_contains!(resp, fixture);
                    });
            });
    }
}

#[test]
#[ignore]
fn nodejs_yarn_1_typescript() {
    test_yarn(
        "yarn-1-typescript",
        vec![
            "Installing yarn",
            "Installing dependencies",
            "Running `build` script",
        ],
    );
}

#[test]
#[ignore]
fn nodejs_yarn_3_modules_nonzero() {
    test_yarn(
        "yarn-3-modules-nonzero",
        vec![
            "Installing yarn",
            "Installing dependencies",
            "Resolution step",
            "Fetch step",
            "Link step",
            "Completed",
        ],
    );
}
