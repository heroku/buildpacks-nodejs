#![warn(clippy::pedantic)]

use libcnb_test::assert_contains;
use test_support::Builder::{Heroku20, Heroku22};
use test_support::{assert_web_response, test_yarn_app};

#[test]
#[ignore]
fn yarn_1_typescript_heroku_20() {
    test_yarn_app("yarn-1-typescript", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Running `build` script");
        assert_web_response(&ctx, "yarn-1-typescript");
    });
}

#[test]
#[ignore]
fn yarn_1_typescript_heroku_22() {
    test_yarn_app("yarn-1-typescript", Heroku22, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Running `build` script");
        assert_web_response(&ctx, "yarn-1-typescript");
    });
}

#[test]
#[ignore]
fn yarn_2_pnp_zero_heroku_20() {
    test_yarn_app("yarn-2-pnp-zero", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Yarn zero-install detected");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-2-pnp-zero");
    });
}

#[test]
#[ignore]
fn yarn_2_pnp_zero_heroku_22() {
    test_yarn_app("yarn-2-pnp-zero", Heroku22, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Yarn zero-install detected");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-2-pnp-zero");
    });
}

#[test]
#[ignore]
fn yarn_2_modules_nonzero_heroku_20() {
    test_yarn_app("yarn-2-modules-nonzero", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Setting up yarn dependency cache");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-2-modules-nonzero");
    });
}

#[test]
#[ignore]
fn yarn_3_pnp_nonzero_heroku_20() {
    test_yarn_app("yarn-3-pnp-nonzero", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Setting up yarn dependency cache");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-3-pnp-nonzero");
    });
}

#[test]
#[ignore]
fn yarn_3_pnp_nonzero_heroku_22() {
    test_yarn_app("yarn-3-pnp-nonzero", Heroku22, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Setting up yarn dependency cache");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-3-pnp-nonzero");
    });
}

#[test]
#[ignore]
fn yarn_3_modules_zero_heroku_20() {
    test_yarn_app("yarn-3-modules-zero", Heroku20, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Yarn zero-install detected");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-3-modules-zero");
    });
}

#[test]
#[ignore]
fn yarn_3_modules_zero_heroku_22() {
    test_yarn_app("yarn-3-modules-zero", Heroku22, |ctx| {
        assert_contains!(ctx.pack_stdout, "Installing yarn");
        assert_contains!(ctx.pack_stdout, "Yarn zero-install detected");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_contains!(ctx.pack_stdout, "Resolution step");
        assert_contains!(ctx.pack_stdout, "Fetch step");
        assert_contains!(ctx.pack_stdout, "Link step");
        assert_contains!(ctx.pack_stdout, "Completed");
        assert_web_response(&ctx, "yarn-3-modules-zero");
    });
}
