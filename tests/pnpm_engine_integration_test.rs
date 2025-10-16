// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::assert_contains;
use test_support::{
    create_build_snapshot, nodejs_integration_test_with_config, set_package_manager,
    set_pnpm_engine,
};

#[test]
#[ignore = "integration test"]
fn pnpm_install_engine() {
    nodejs_integration_test_with_config(
        "./fixtures/pnpm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_pnpm_engine(&app_dir, "7.32.3");
                std::fs::rename(
                    app_dir.join("pnpm-lock.v7.yaml"),
                    app_dir.join("pnpm-lock.yaml"),
                )
                .unwrap();
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            let output = ctx.run_shell_command("pnpm --version");
            assert_contains!(output.stdout, "7.32.3");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn pnpm_install_package_manager() {
    nodejs_integration_test_with_config(
        "./fixtures/pnpm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "pnpm@8.4.0");
                std::fs::rename(
                    app_dir.join("pnpm-lock.v8.yaml"),
                    app_dir.join("pnpm-lock.yaml"),
                )
                .unwrap();
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            let output = ctx.run_shell_command("pnpm --version");
            assert_contains!(output.stdout, "8.4.0");
        },
    );
}
