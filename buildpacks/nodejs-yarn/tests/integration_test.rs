#![warn(clippy::pedantic)]

use libcnb_test::{assert_contains, assert_not_contains};
use test_support::{assert_web_response, nodejs_integration_test};

#[test]
#[ignore = "integration test"]
fn yarn_1_typescript() {
    nodejs_integration_test("./fixtures/yarn-1-typescript", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing Node");
        assert_contains!(ctx.pack_stdout, "Installing yarn CLI");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Running `build` script");

        assert_not_contains!(ctx.pack_stdout, "corepack");
        assert_not_contains!(
            ctx.pack_stdout,
            "Installing node modules from ./package-lock.json"
        );

        assert_web_response(&ctx, "yarn-1-typescript");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_2_pnp_zero() {
    nodejs_integration_test("./fixtures/yarn-2-pnp-zero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing Node");
        assert_contains!(ctx.pack_stdout, "Installing yarn 2.4.1 via corepack");
        assert_contains!(ctx.pack_stdout, "Yarn zero-install detected");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");

        assert_not_contains!(ctx.pack_stdout, "Installing yarn CLI");
        assert_not_contains!(
            ctx.pack_stdout,
            "Installing node modules from ./package-lock.json"
        );
        assert_not_contains!(
            ctx.pack_stdout,
            "can't be found in the cache and will be fetched from the remote registry"
        );

        assert_web_response(&ctx, "yarn-2-pnp-zero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_2_modules_nonzero() {
    nodejs_integration_test("./fixtures/yarn-2-modules-nonzero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Successfully set cacheFolder");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(
            ctx.pack_stdout,
            "can't be found in the cache and will be fetched from the remote registry"
        );
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-2-modules-nonzero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_3_pnp_nonzero() {
    nodejs_integration_test("./fixtures/yarn-3-pnp-nonzero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Successfully set cacheFolder");
        assert_contains!(
            ctx.pack_stdout,
            "can't be found in the cache and will be fetched from the remote registry"
        );
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-3-pnp-nonzero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_3_modules_zero() {
    nodejs_integration_test("./fixtures/yarn-3-modules-zero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Yarn zero-install detected");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_not_contains!(
            ctx.pack_stdout,
            "can't be found in the cache and will be fetched from the remote registry"
        );
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-3-modules-zero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_4_pnp_nonzero() {
    nodejs_integration_test("./fixtures/yarn-4-pnp-nonzero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(
            ctx.pack_stdout,
            "Successfully set enableGlobalCache to false"
        );
        assert_contains!(ctx.pack_stdout, "Successfully set cacheFolder");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "62 packages were added to the project");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-4-pnp-nonzero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_4_modules_zero() {
    nodejs_integration_test("./fixtures/yarn-4-modules-zero", |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(
            ctx.pack_stdout,
            "Successfully set enableGlobalCache to false"
        );
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_not_contains!(ctx.pack_stdout, "packages were added to the project");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-4-modules-zero");
    });
}
