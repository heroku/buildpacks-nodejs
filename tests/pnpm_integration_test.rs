// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{BuildpackReference, assert_contains};
use test_support::{
    add_build_script, assert_web_response, create_build_snapshot, custom_buildpack,
    integration_test_with_config, nodejs_integration_test, nodejs_integration_test_with_config,
    set_package_manager, set_pnpm_engine,
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

#[test]
#[ignore = "integration test"]
fn pnpm_7_pnp() {
    nodejs_integration_test("./fixtures/pnpm-7-pnp", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "pnpm-7-pnp");
    });
}

#[test]
#[ignore = "integration test"]
fn pnpm_8_hoist() {
    nodejs_integration_test("./fixtures/pnpm-8-hoist", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "pnpm-8-hoist");
    });
}

#[test]
#[ignore = "integration test"]
fn pnpm_8_nuxt() {
    nodejs_integration_test("./fixtures/pnpm-8-nuxt", |ctx| {
        create_build_snapshot(&ctx.pack_stdout)
            .filter(
                r"( *)> nuxt build\n(?:.*\n)*? *\[nitro] ✔ You can preview this build.*",
                "${1}> nuxt build\n\n${1}<NUXT BUILD OUTPUT>",
            )
            .filter(
                r"( *)\.\.\./esbuild@\d+\.\d+\.\d+/node_modules/esbuild postinstall.*",
                "${1}<ESBUILD POSTINSTALL_SCRIPT>",
            )
            .filter(
                r"( *)ERROR  \(node:\d+\).*\n *\(Use node --trace-deprecation.*",
                "${1}<NODE DEPRECATION ERROR>",
            )
            .assert();
    });
}

#[test]
#[ignore = "integration test"]
fn test_pnpm_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test("./fixtures/pnpm-project-with-native-module", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_pnpm_skip_build_scripts_from_buildplan() {
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
                            name = "heroku/nodejs"
                            [requires.metadata]
                            enabled = false
                            skip_pruning = true
                        EOF
                    "# })
                    .call(),
            ),
        ],
    );
}

#[test]
#[ignore = "integration test"]
fn test_pnpm_default_web_process_registration_is_skipped_if_procfile_exists() {
    nodejs_integration_test_with_config(
        "./fixtures/pnpm-9",
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
fn test_pnpm_prune_dev_dependencies_config() {
    nodejs_integration_test_with_config(
        "./fixtures/pnpm-9",
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
