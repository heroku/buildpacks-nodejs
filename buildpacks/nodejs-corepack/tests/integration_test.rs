#![warn(clippy::pedantic)]

use libcnb_test::assert_contains;
use test_support::test_corepack_app;
use test_support::Builder::{Heroku20, Heroku22};

#[test]
#[ignore = "integration test"]
fn corepack_yarn_2_heroku_20() {
    test_corepack_app("yarn-2-pnp-zero", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing yarn@2.4.1");
        let output = ctx.run_shell_command("yarn --version");
        assert_contains!(output.stdout, "2.4.1");
    });
}

#[test]
#[ignore = "integration test"]
fn corepack_yarn_3_heroku_22() {
    test_corepack_app("yarn-3-pnp-nonzero", Heroku22, |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing yarn@3.2.0");
        let output = ctx.run_shell_command("yarn --version");
        assert_contains!(output.stdout, "3.2.0");
    });
}

#[test]
#[ignore = "integration test"]
fn corepack_pnpm_7() {
    test_corepack_app("pnpm-7-pnp", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing pnpm@7.32.3");
        let output = ctx.run_shell_command("pnpm --version");
        assert_contains!(output.stdout, "7.32.3");
    });
}

#[test]
#[ignore = "integration test"]
fn corepack_pnpm_8() {
    test_corepack_app("pnpm-8-hoist", Heroku22, |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing pnpm@8.4.0");
        let output = ctx.run_shell_command("pnpm --version");
        assert_contains!(output.stdout, "8.4.0");
    });
}
