#![warn(clippy::pedantic)]

use libcnb_test::assert_contains;
use std::fs;
use test_support::{nodejs_integration_test_with_config, set_package_manager};

#[test]
#[ignore = "integration test"]
fn corepack_pnpm() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-with-pnpm",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "pnpm@7.32.3");
                fs::rename(
                    app_dir.join("pnpm-lock.v7.32.3.yaml"),
                    app_dir.join("pnpm-lock.yaml"),
                )
                .unwrap();
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Preparing pnpm@7.32.3");
            let output = ctx.run_shell_command("pnpm --version");
            assert_contains!(output.stdout, "7.32.3");

            let mut config = ctx.config.clone();
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "pnpm@8.4.0");
                fs::rename(
                    app_dir.join("pnpm-lock.v8.4.0.yaml"),
                    app_dir.join("pnpm-lock.yaml"),
                )
                .unwrap();
            });

            ctx.rebuild(config, |ctx| {
                assert_contains!(ctx.pack_stdout, "Preparing pnpm@8.4.0");
                let output = ctx.run_shell_command("pnpm --version");
                assert_contains!(output.stdout, "8.4.0");
            });
        },
    );
}

#[test]
#[ignore = "integration test"]
fn corepack_yarn() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-with-yarn",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "yarn@2.4.1");
                fs::rename(app_dir.join("yarn.v2.4.1.lock"), app_dir.join("yarn.lock")).unwrap();
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Preparing yarn@2.4.1");
            let output = ctx.run_shell_command("yarn --version");
            assert_contains!(output.stdout, "2.4.1");

            let mut config = ctx.config.clone();
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "yarn@3.2.0");
                fs::rename(app_dir.join("yarn.v3.2.0.lock"), app_dir.join("yarn.lock")).unwrap();
            });

            ctx.rebuild(config, |ctx| {
                assert_contains!(ctx.pack_stdout, "Preparing yarn@3.2.0");
                let output = ctx.run_shell_command("yarn --version");
                assert_contains!(output.stdout, "3.2.0");
            });
        },
    );
}
