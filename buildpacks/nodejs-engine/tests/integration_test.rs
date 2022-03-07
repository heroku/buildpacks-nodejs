#[cfg(test)]
use libcnb_test::{BuildpackReference, IntegrationTest};
use std::time::Duration;
use ureq;

#[test]
fn test_node_with_indexjs() {
    IntegrationTest::new(
        "heroku/buildpacks:20",
        "../../test/fixtures/node-with-indexjs",
    )
    .run_test(|ctx| {
        println!("{}", ctx.pack_stdout);
        assert!(ctx
            .pack_stdout
            .contains("Detected Node.js version range: *"));
        assert!(ctx.pack_stdout.contains("Installing Node.js"));
        let port = 8080;
        ctx.start_container(&[port], |container| {
            std::thread::sleep(Duration::from_secs(1));
            let addr = container
                .address_for_port(port)
                .expect("couldn't get container address");
            let resp = ureq::get(&format!("http://{addr}"))
                .call()
                .expect("request to container failed")
                .into_string()
                .expect("response read error");
            assert!(resp.contains("node-with-indexjs"))
        })
    });
}

#[test]
fn test_node_with_serverjs() {
    IntegrationTest::new(
        "heroku/buildpacks:20",
        "../../test/fixtures/node-with-serverjs",
    )
    .buildpacks(vec![BuildpackReference::Crate])
    .run_test(|ctx| {
        println!("{}", ctx.pack_stdout);
        assert!(ctx.pack_stdout.contains("Installing Node.js 16.0.0"));
        let port = 8080;
        ctx.start_container(&[port], |container| {
            std::thread::sleep(Duration::from_secs(1));
            let addr = container
                .address_for_port(port)
                .expect("couldn't get container address");
            let resp = ureq::get(&format!("http://{addr}"))
                .call()
                .expect("request to container failed")
                .into_string()
                .expect("response read error");
            assert!(resp.contains("node-with-serverjs"))
        })
    });
}
