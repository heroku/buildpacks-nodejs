// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::BuildpackReference;
use test_support::{
    add_build_script, assert_web_response, create_build_snapshot, custom_buildpack,
    integration_test_with_config, nodejs_integration_test, nodejs_integration_test_with_config,
};

#[test]
#[ignore = "integration test"]
fn yarn_1_typescript() {
    nodejs_integration_test("./fixtures/yarn-1-typescript", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-1-typescript");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_2_pnp_zero() {
    nodejs_integration_test("./fixtures/yarn-2-pnp-zero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-2-pnp-zero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_2_modules_nonzero() {
    nodejs_integration_test("./fixtures/yarn-2-modules-nonzero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-2-modules-nonzero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_3_pnp_nonzero() {
    nodejs_integration_test("./fixtures/yarn-3-pnp-nonzero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-3-pnp-nonzero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_3_modules_zero() {
    nodejs_integration_test("./fixtures/yarn-3-modules-zero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-3-modules-zero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_4_pnp_nonzero() {
    nodejs_integration_test("./fixtures/yarn-4-pnp-nonzero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-4-pnp-nonzero");
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_4_modules_zero() {
    nodejs_integration_test("./fixtures/yarn-4-modules-zero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-4-modules-zero");
    });
}

#[test]
#[ignore = "integration test"]
fn test_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test("./fixtures/yarn-project-with-native-module", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
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
            create_build_snapshot(&ctx.pack_stdout).assert();
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
            create_build_snapshot(&ctx.pack_stdout).assert();
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_prune_dev_dependencies_config() {
    nodejs_integration_test_with_config(
        "./fixtures/yarn-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                std::fs::write(
                    app_dir.join("project.toml"),
                    indoc! { "
                    [com.heroku.buildpacks.nodejs]
                    actions.prune_dev_dependencies = false
                " },
                )
                .unwrap();
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
        },
    );
}
