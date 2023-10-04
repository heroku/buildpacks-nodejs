#![warn(clippy::pedantic)]

use libcnb_test::assert_contains;
use test_support::{nodejs_integration_test_with_config, set_package_manager};

#[test]
#[ignore = "integration test"]
fn corepack_yarn_2() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-template",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "yarn@2.4.1");
                std::fs::rename(app_dir.join("yarn.v2.lock"), app_dir.join("yarn.lock")).unwrap();
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Preparing yarn@2.4.1");
            let output = ctx.run_shell_command("yarn --version");
            assert_contains!(output.stdout, "2.4.1");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn corepack_yarn_3() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-template",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "yarn@3.2.0");
                std::fs::rename(app_dir.join("yarn.v3.lock"), app_dir.join("yarn.lock")).unwrap();
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Preparing yarn@3.2.0");
            let output = ctx.run_shell_command("yarn --version");
            assert_contains!(output.stdout, "3.2.0");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn corepack_pnpm_7() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-template",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "pnpm@7.32.3");
                std::fs::rename(
                    app_dir.join("pnpm-lock.v7.yaml"),
                    app_dir.join("pnpm-lock.yaml"),
                )
                .unwrap();
            });
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "Preparing pnpm@7.32.3");
            let output = ctx.run_shell_command("pnpm --version");
            assert_contains!(output.stdout, "7.32.3");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn corepack_pnpm_8() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-template",
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
            assert_contains!(ctx.pack_stdout, "Preparing pnpm@8.4.0");
            let output = ctx.run_shell_command("pnpm --version");
            assert_contains!(output.stdout, "8.4.0");
        },
    );
}
