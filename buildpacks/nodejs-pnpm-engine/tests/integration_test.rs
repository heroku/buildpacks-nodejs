// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::formatdoc;
use libcnb_test::{assert_contains, PackResult};
use test_support::nodejs_integration_test_with_config;

#[test]
#[ignore = "integration test"]
fn pnpm_unknown_version() {
    nodejs_integration_test_with_config(
        "./fixtures/pnpm-unknown-version",
        |cfg| {
            cfg.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stderr,
                &formatdoc! {"
                    ! A pnpm lockfile (`pnpm-lock.yaml`) was detected, but the \
                    version of `pnpm` to install could not be determined.
                    !
                    ! `pnpm` may be installed via the `heroku/nodejs-corepack` \
                    buildpack. It requires the desired `pnpm` version to be set \
                    via the `packageManager` key in `package.json`.
                    !
                    ! To set `packageManager` in `package.json` to the latest \
                    `pnpm`, run:
                    !
                    ! `corepack enable`
                    ! `corepack use pnpm@*`
                    !
                    ! Then commit the result, and try again.
                "}
            );
        },
    );
}
