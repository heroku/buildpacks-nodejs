// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb_test::assert_contains;
use test_support::{
    create_build_snapshot, nodejs_integration_test_with_config, set_package_manager,
};

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
            create_build_snapshot(&ctx.pack_stdout).assert();
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
            create_build_snapshot(&ctx.pack_stdout).assert();
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
            create_build_snapshot(&ctx.pack_stdout).assert();
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
            create_build_snapshot(&ctx.pack_stdout).assert();
            let output = ctx.run_shell_command("pnpm --version");
            assert_contains!(output.stdout, "8.4.0");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn corepack_npm_8() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-template",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "npm@8.19.4");
                std::fs::rename(
                    app_dir.join("package-lock.v8.json"),
                    app_dir.join("package-lock.json"),
                )
                .unwrap();
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            let output = ctx.run_shell_command("npm --version");
            assert_contains!(output.stdout, "8.19.4");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn corepack_npm_10() {
    nodejs_integration_test_with_config(
        "./fixtures/corepack-template",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                set_package_manager(&app_dir, "npm@10.2.0");
                std::fs::rename(
                    app_dir.join("package-lock.v10.json"),
                    app_dir.join("package-lock.json"),
                )
                .unwrap();
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            let output = ctx.run_shell_command("npm --version");
            assert_contains!(output.stdout, "10.2.0");
        },
    );
}
