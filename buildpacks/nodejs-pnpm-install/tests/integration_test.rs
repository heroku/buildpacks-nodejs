// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::formatdoc;
use libcnb_test::{assert_contains, assert_empty};
use test_support::{assert_web_response, nodejs_integration_test};

#[test]
#[ignore = "integration test"]
fn pnpm_7_pnp() {
    nodejs_integration_test("./fixtures/pnpm-7-pnp", |ctx| {
        assert_empty!(ctx.pack_stderr);
        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Setting up pnpm dependency store]
                Creating new pnpm content-addressable store
                Creating pnpm virtual store
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Installing dependencies]
                Lockfile is up to date, resolution step is skipped
                Progress: resolved 1, reused 0, downloaded 0, added 0
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                Packages: +60
                ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                Packages are hard linked from the content-addressable store to the virtual store.
                  Content-addressable store is at: /layers/heroku_nodejs-pnpm-install/addressable/v3
                  Virtual store is at:             ../layers/heroku_nodejs-pnpm-install/virtual/store
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Running scripts]
                No build scripts found
            "}
        );
        assert_web_response(&ctx, "pnpm-7-pnp");
    });
}

#[test]
#[ignore = "integration test"]
fn pnpm_8_hoist() {
    nodejs_integration_test("./fixtures/pnpm-8-hoist", |ctx| {
        assert_empty!(ctx.pack_stderr);
        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Setting up pnpm dependency store]
                Creating new pnpm content-addressable store
                Creating pnpm virtual store
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Installing dependencies]
                Lockfile is up to date, resolution step is skipped
                Progress: resolved 1, reused 0, downloaded 0, added 0
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                Packages: +57
                +++++++++++++++++++++++++++++++++++++++++++++++++++++++++
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                Packages are hard linked from the content-addressable store to the virtual store.
                  Content-addressable store is at: /layers/heroku_nodejs-pnpm-install/addressable/v3
                  Virtual store is at:             ../layers/heroku_nodejs-pnpm-install/virtual/store
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Running scripts]
                No build scripts found
            "}
        );
        assert_web_response(&ctx, "pnpm-8-hoist");
    });
}

#[test]
#[ignore = "integration test"]
fn pnpm_8_nuxt() {
    nodejs_integration_test("./fixtures/pnpm-8-nuxt", |ctx| {
        assert_empty!(ctx.pack_stderr);
        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Setting up pnpm dependency store]
                Creating new pnpm content-addressable store
                Creating pnpm virtual store
            "}
        );

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Installing dependencies]
                Lockfile is up to date, resolution step is skipped
                Progress: resolved 1, reused 0, downloaded 0, added 0
            "}
        );

        assert_contains!(ctx.pack_stdout, "Packages: +676");

        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                [Running scripts]
                Running `build` script
            "}
        );
    });
}
