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
                            std::thread::sleep(Duration::from_secs(1));
                            let addr = container
                                .address_for_port(port)
                                .expect("couldn't get container address");
                            let resp = ureq::get(&format!("http://{addr}"))
                                .call()
                                .expect("request to container failed")
                                .into_string()
                                .expect("response read error");

                            assert_contains!(resp, "node-with-index");
                        },
                    );
                },
            );
        },
    );
}
