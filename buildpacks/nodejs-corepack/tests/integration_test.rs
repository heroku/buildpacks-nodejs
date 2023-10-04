#![warn(clippy::pedantic)]

use libcnb_test::assert_contains;
use test_support::nodejs_integration_test;

#[test]
#[ignore = "integration test"]
fn corepack_yarn_2() {
    nodejs_integration_test("../../../test/fixtures/yarn-2-pnp-zero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing yarn@2.4.1");
        let output = ctx.run_shell_command("yarn --version");
        assert_contains!(output.stdout, "2.4.1");
    });
}

#[test]
#[ignore = "integration test"]
fn corepack_yarn_3() {
    nodejs_integration_test("../../../test/fixtures/yarn-3-pnp-nonzero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing yarn@3.2.0");
        let output = ctx.run_shell_command("yarn --version");
        assert_contains!(output.stdout, "3.2.0");
    });
}

#[test]
#[ignore = "integration test"]
fn corepack_pnpm_7() {
    nodejs_integration_test("../../../test/fixtures/pnpm-7-pnp", |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing pnpm@7.32.3");
        let output = ctx.run_shell_command("pnpm --version");
        assert_contains!(output.stdout, "7.32.3");
    });
}

#[test]
#[ignore = "integration test"]
fn corepack_pnpm_8() {
    nodejs_integration_test("../../../test/fixtures/pnpm-8-hoist", |ctx| {
        assert_contains!(ctx.pack_stdout, "Preparing pnpm@8.4.0");
        let output = ctx.run_shell_command("pnpm --version");
        assert_contains!(output.stdout, "8.4.0");
    });
}
