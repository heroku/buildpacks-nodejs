#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, BuildConfig, ContainerConfig, TestRunner};
use std::time::Duration;

fn test_node(fixture: &str, builder: &str, expect_lines: &[&str]) {
    TestRunner::default().build(
        BuildConfig::new(builder, format!("../../test/fixtures/{fixture}")),
        |ctx| {
            for expect_line in expect_lines {
                assert_contains!(ctx.pack_stdout, expect_line);
            }
            let port = 8080;
            ctx.start_container(ContainerConfig::new().expose_port(port), |container| {
                std::thread::sleep(Duration::from_secs(1));
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
        },
    );
}

#[test]
#[ignore]
fn simple_indexjs_heroku20() {
    test_node(
        "node-with-indexjs",
        "heroku/buildpacks:20",
        &["Detected Node.js version range: *", "Installing Node.js"],
    );
}

#[test]
#[ignore]
fn simple_indexjs_heroku22() {
    test_node(
        "node-with-indexjs",
        "heroku/builder:22",
        &["Detected Node.js version range: *", "Installing Node.js"],
    );
}

#[test]
#[ignore]
fn simple_serverjs_heroku20() {
    test_node(
        "node-with-serverjs",
        "heroku/buildpacks:20",
        &["Installing Node.js 16.0.0"],
    );
}

#[test]
#[ignore]
fn simple_serverjs_heroku22() {
    test_node(
        "node-with-serverjs",
        "heroku/builder:22",
        &["Installing Node.js 16.0.0"],
    );
}
