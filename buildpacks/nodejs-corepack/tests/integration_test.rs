#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, ContainerConfig};
use test_support::test_corepack_app;
use test_support::Builder::{Heroku20, Heroku22};

#[test]
#[ignore = "integration test"]
fn corepack_yarn_2_heroku_20() {
    test_corepack_app("yarn-2-pnp-zero", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing yarn@2.4.1");
        ctx.start_container(
            ContainerConfig::new()
                .entrypoint(["launcher"])
                .command(["yarn", "--version"]),
            |ctr| {
                let logs = ctr.logs_wait();
                assert_contains!(logs.stdout, "2.4.1");
            },
        );
    });
}
#[test]
#[ignore = "integration test"]
fn corepack_yarn_3_heroku_22() {
    test_corepack_app("yarn-3-pnp-nonzero", Heroku22, |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing yarn@3.2.0");
        ctx.start_container(
            ContainerConfig::new()
                .entrypoint(["launcher"])
                .command(["yarn", "--version"]),
            |ctr| {
                let logs = ctr.logs_wait();
                assert_contains!(logs.stdout, "3.2.0");
            },
        );
    });
}
