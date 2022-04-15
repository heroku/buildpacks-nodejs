#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, BuildpackReference, IntegrationTest};
use std::time::Duration;

#[test]
#[ignore]
fn nodejs_yarn_1_typescript() {
    IntegrationTest::new(
        "heroku/buildpacks:22",
        "../../test/fixtures/yarn-1-typescript",
    )
    .buildpacks(vec![
        BuildpackReference::Other(String::from("heroku/nodejs-engine")),
        BuildpackReference::Crate,
    ])
    .run_test(|ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Running `build` script");
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

                assert_contains!(resp, "yarn-1-typescript");
            });
    });
}
