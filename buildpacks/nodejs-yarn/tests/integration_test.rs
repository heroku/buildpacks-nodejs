// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{assert_contains, assert_not_contains, BuildpackReference};
use test_support::{
    add_build_script, assert_web_response, custom_buildpack, integration_test_with_config,
    nodejs_integration_test, nodejs_integration_test_with_config,
};

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
        assert_contains!(ctx.pack_stdout, "Installing `yarn@2.4.1`");
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
        assert_contains!(ctx.pack_stdout, "Installing `yarn@3");
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
        assert_contains!(ctx.pack_stdout, "Installing `yarn@3");
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
        assert_contains!(ctx.pack_stdout, "Installing `yarn@4");
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
        assert_contains!(ctx.pack_stdout, "Installing `yarn@4");
        assert_contains!(ctx.pack_stdout, "Installing dependencies");
        assert_not_contains!(ctx.pack_stdout, "Successfully set cacheFolder");
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

#[test]
#[ignore = "integration test"]
fn test_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test("./fixtures/yarn-project-with-native-module", |ctx| {
        assert_not_contains!(ctx.pack_stdout, "Restoring yarn dependency cache");
        assert_contains!(ctx.pack_stdout, "dtrace-provider@npm:0.8.8 must be built because it never has been before or the last one failed");
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            assert_contains!(ctx.pack_stdout, "Restoring yarn dependency cache");
            assert_contains!(ctx.pack_stdout, "dtrace-provider@npm:0.8.8 must be built because it never has been before or the last one failed");
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_skip_build_scripts_from_buildplan() {
    integration_test_with_config(
        "./fixtures/yarn-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                add_build_script(&app_dir, "heroku-prebuild");
                add_build_script(&app_dir, "build");
                add_build_script(&app_dir, "heroku-postbuild");
            });
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "! Not running `heroku-prebuild` as it was disabled by a participating buildpack"
            );
            assert_contains!(
                ctx.pack_stdout,
                "! Not running `build` as it was disabled by a participating buildpack"
            );
            assert_contains!(
                ctx.pack_stdout,
                "! Not running `heroku-postbuild` as it was disabled by a participating buildpack"
            );
        },
        &[
            BuildpackReference::WorkspaceBuildpack(buildpack_id!("heroku/nodejs")),
            BuildpackReference::Other(
                custom_buildpack()
                    .id("test/skip-build-scripts")
                    .detect(indoc! { r#"
                        #!/usr/bin/env bash
                        
                        build_plan="$2"
                        
                        cat <<EOF >"$build_plan"
                            [[requires]]
                            name = "node_build_scripts"
                            [requires.metadata]
                            enabled = false
                        EOF
                    "# })
                    .call(),
            ),
        ],
    );
}

#[test]
#[ignore = "integration test"]
fn test_default_web_process_registration_is_skipped_if_procfile_exists() {
    nodejs_integration_test_with_config(
        "./fixtures/yarn-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                std::fs::File::create(app_dir.join("Procfile")).unwrap();
            });
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                "Skipping default web process (Procfile detected)"
            );
        },
    );
}
