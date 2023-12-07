// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::formatdoc;
use libcnb_test::{assert_contains, assert_empty};
use test_support::{assert_web_response, nodejs_integration_test};

#[test]
#[ignore = "integration test"]
fn pnpm_unknown_version() {
    nodejs_integration_test("./fixtures/pnpm-unknown-version", |ctx| {
        assert_empty!(ctx.pack_stderr);
        assert_contains!(
            ctx.pack_stdout,
            &formatdoc! {"
                Use corepack instead!
            "}
        );
    });
}
