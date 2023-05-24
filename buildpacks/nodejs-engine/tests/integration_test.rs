#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, BuildConfig, ContainerConfig, ContainerContext, TestRunner};
use std::time::Duration;

const PORT: u16 = 8080;

fn test_node(fixture: &str, builder: &str, expect_lines: &[&str]) {
    TestRunner::default().build(
        BuildConfig::new(builder, format!("../../test/fixtures/{fixture}")),
        |ctx| {
            for expect_line in expect_lines {
                assert_contains!(ctx.pack_stdout, expect_line);
            }
            ctx.start_container(ContainerConfig::new().expose_port(PORT), |container| {
                test_response(&container, fixture);
            });
        },
    );
}

fn test_response(container: &ContainerContext, text: &str) {
    std::thread::sleep(Duration::from_secs(5));
    let addr = container
        .address_for_port(PORT)
        .expect("couldn't get container address");
    let resp = ureq::get(&format!("http://{addr}"))
        .call()
        .expect("request to container failed")
        .into_string()
        .expect("response read error");

    assert_contains!(resp, text);
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

#[test]
#[ignore]
fn upgrade_simple_indexjs_from_heroku20_to_heroku22() {
    TestRunner::default().build(
        BuildConfig::new(
            "heroku/buildpacks:20",
            "../../test/fixtures/node-with-indexjs",
        ),
        |initial_ctx| {
            assert_contains!(initial_ctx.pack_stdout, "Installing Node.js");
            initial_ctx.rebuild(
                BuildConfig::new("heroku/builder:22", "../../test/fixtures/node-with-indexjs"),
                |upgrade_ctx| {
                    assert_contains!(upgrade_ctx.pack_stdout, "Installing Node.js");
                    let port = 8080;
                    upgrade_ctx.start_container(
                        ContainerConfig::new().expose_port(port),
                        |container| {
                            test_response(&container, "node-with-index");
                        },
                    );
                },
            );
        },
    );
}
