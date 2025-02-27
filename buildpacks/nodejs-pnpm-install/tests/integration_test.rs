// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::{formatdoc, indoc};
use libcnb::data::buildpack_id;
use libcnb_test::{assert_contains, BuildpackReference};
use test_support::{
    add_build_script, assert_web_response, custom_buildpack, integration_test_with_config,
    nodejs_integration_test, nodejs_integration_test_with_config,
};

#[test]
#[ignore = "integration test"]
fn pnpm_7_pnp() {
    nodejs_integration_test("./fixtures/pnpm-7-pnp", |ctx| {
        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Setting up pnpm dependency store
                  - Creating new pnpm content-addressable store
                  - Creating pnpm virtual store
            "}
        );

        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Installing dependencies
                  - Running `pnpm install --frozen-lockfile`
            "}
        );

        assert_contains!(
            ctx.pack_stderr,
            "Packages are hard linked from the content-addressable store to the virtual store."
        );
        assert_contains!(
            ctx.pack_stderr,
            "Content-addressable store is at: /layers/heroku_nodejs-pnpm-install/addressable/v3"
        );
        assert_contains!(
            ctx.pack_stderr,
            "Virtual store is at:             ../layers/heroku_nodejs-pnpm-install/virtual/store"
        );

        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Running scripts
                  - No build scripts found
            "}
        );
        assert_web_response(&ctx, "pnpm-7-pnp");
    });
}

#[test]
#[ignore = "integration test"]
fn pnpm_8_hoist() {
    nodejs_integration_test("./fixtures/pnpm-8-hoist", |ctx| {
        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Setting up pnpm dependency store
                  - Creating new pnpm content-addressable store
                  - Creating pnpm virtual store
            "}
        );

        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Installing dependencies
                  - Running `pnpm install --frozen-lockfile`
            "}
        );

        assert_contains!(
            ctx.pack_stderr,
            "Packages are hard linked from the content-addressable store to the virtual store."
        );
        assert_contains!(
            ctx.pack_stderr,
            "Content-addressable store is at: /layers/heroku_nodejs-pnpm-install/addressable/v3"
        );
        assert_contains!(
            ctx.pack_stderr,
            "Virtual store is at:             ../layers/heroku_nodejs-pnpm-install/virtual/store"
        );

        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Running scripts
                  - No build scripts found
            "}
        );
        assert_web_response(&ctx, "pnpm-8-hoist");
    });
}

#[test]
#[ignore = "integration test"]
fn pnpm_8_nuxt() {
    nodejs_integration_test("./fixtures/pnpm-8-nuxt", |ctx| {
        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Setting up pnpm dependency store
                  - Creating new pnpm content-addressable store
                  - Creating pnpm virtual store
            "}
        );

        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Installing dependencies
                  - Running `pnpm install --frozen-lockfile`
            "}
        );

        assert_contains!(
            ctx.pack_stderr,
            &formatdoc! {"
                - Running scripts
                  - Running `build` script
            "}
        );
    });
}

#[test]
#[ignore = "integration test"]
fn test_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test("./fixtures/pnpm-project-with-native-module", |ctx| {
        assert_contains!(
            ctx.pack_stderr,
            "Creating new pnpm content-addressable store"
        );
        assert_contains!(ctx.pack_stderr, "dtrace-provider install");
        assert_contains!(ctx.pack_stderr, "node-gyp rebuild");
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            assert_contains!(
                ctx.pack_stderr,
                "Restoring pnpm content-addressable store from cache"
            );
            assert_contains!(ctx.pack_stderr, "dtrace-provider install");
            assert_contains!(ctx.pack_stderr, "node-gyp rebuild");
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_skip_build_scripts_from_buildplan() {
    integration_test_with_config(
        "./fixtures/pnpm-9",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                add_build_script(&app_dir, "heroku-prebuild");
                add_build_script(&app_dir, "build");
                add_build_script(&app_dir, "heroku-postbuild");
            });
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stderr,
                "! Not running `heroku-prebuild` as it was disabled by a participating buildpack"
            );
            assert_contains!(
                ctx.pack_stderr,
                "! Not running `build` as it was disabled by a participating buildpack"
            );
            assert_contains!(
                ctx.pack_stderr,
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
        "./fixtures/pnpm-9",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                std::fs::File::create(app_dir.join("Procfile")).unwrap();
            });
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stderr,
                "Skipping default web process (Procfile detected)"
            );
        },
    );
}
