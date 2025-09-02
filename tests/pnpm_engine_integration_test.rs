// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::PackResult;
use test_support::{create_build_snapshot, nodejs_integration_test_with_config};

#[test]
#[ignore = "integration test"]
fn pnpm_unknown_version() {
    nodejs_integration_test_with_config(
        "./fixtures/pnpm-unknown-version",
        |cfg| {
            cfg.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
        },
    );
}
