// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{BuildpackReference, TestContext, assert_contains, assert_not_contains};
use std::os::unix::fs::PermissionsExt;
use test_support::{
    add_build_script, assert_web_response, create_build_snapshot, custom_buildpack,
    integration_test_with_config, nodejs_integration_test, nodejs_integration_test_with_config,
    print_build_env_buildpack,
};

#[test]
#[ignore = "integration test"]
fn yarn_install() {
    integration_test_with_config(
        "./fixtures/yarn-project",
        |_| {},
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            assert_contains!(
                ctx.run_shell_command("env").stdout,
                "PATH=/workspace/node_modules/.bin:/layers/heroku_nodejs/yarn/bin:/layers/heroku_nodejs/dist/bin"
            );
        },
        &[
            BuildpackReference::WorkspaceBuildpack(buildpack_id!("heroku/nodejs")),
            print_build_env_buildpack(),
        ],
    );
}

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
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "yarn-2-modules-nonzero");
        assert_yarn_berry_runtime(&ctx, Linker::NodeModules);

        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-2-modules-nonzero");
            assert_yarn_berry_runtime(&ctx, Linker::NodeModules);
        });
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_3_pnp_nonzero() {
    nodejs_integration_test("./fixtures/yarn-3-pnp-nonzero", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "yarn-3-pnp-nonzero");
        assert_yarn_berry_runtime(&ctx, Linker::Pnp);

        let mut config = ctx.config.clone();
        config.env(UNREACHABLE_REGISTRY.0, UNREACHABLE_REGISTRY.1);
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-3-pnp-nonzero");
            assert_yarn_berry_runtime(&ctx, Linker::Pnp);
        });
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
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "yarn-4-pnp-nonzero");
        // The buildpack configures Yarn with env vars, so the app's config is left unmodified
        assert_eq!(
            ctx.run_shell_command("cat /workspace/.yarnrc.yml").stdout,
            indoc! {"
                nodeLinker: pnp

                yarnPath: .yarn/releases/yarn-4.0.0.cjs
            "}
        );
        assert_yarn_berry_runtime(&ctx, Linker::Pnp);
        assert_yarn_4_pnp_nonzero_unplugged_packages(&ctx);

        let mut config = ctx.config.clone();
        config.env(UNREACHABLE_REGISTRY.0, UNREACHABLE_REGISTRY.1);
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-4-pnp-nonzero");
            assert_yarn_berry_runtime(&ctx, Linker::Pnp);
            assert_yarn_4_pnp_nonzero_unplugged_packages(&ctx);
        });
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_4_pnp_prune_dev_dependencies_disabled() {
    nodejs_integration_test_with_config(
        "./fixtures/yarn-4-pnp-nonzero",
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
            assert_web_response(&ctx, "yarn-4-pnp-nonzero");
            // dev dependencies are kept in the cache shipped at launch
            let yarn_cache = ctx
                .run_shell_command("ls /layers/heroku_nodejs/yarn_cache")
                .stdout;
            assert_contains!(yarn_cache, "express-npm-");
            assert_contains!(yarn_cache, "dotenv-npm-");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn yarn_4_pnp_workspaces() {
    nodejs_integration_test("./fixtures/yarn-4-pnp-workspaces", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "Hello from yarn-4-pnp-workspaces");
        assert_yarn_berry_runtime(&ctx, Linker::Pnp);
        assert_yarn_4_pnp_workspace_dependencies(&ctx);

        let mut config = ctx.config.clone();
        config.env(UNREACHABLE_REGISTRY.0, UNREACHABLE_REGISTRY.1);
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "Hello from yarn-4-pnp-workspaces");
            assert_yarn_berry_runtime(&ctx, Linker::Pnp);
            assert_yarn_4_pnp_workspace_dependencies(&ctx);
        });
    });
}

/// Rebuilds with this registry only pass if every package is restored from the Yarn mirror.
const UNREACHABLE_REGISTRY: (&str, &str) = ("YARN_NPM_REGISTRY_SERVER", "http://127.0.0.1:9");

fn assert_yarn_4_pnp_nonzero_unplugged_packages(ctx: &TestContext) {
    // `depd` (prod) and `picocolors` (dev) are unplugged with `dependenciesMeta`
    let unplugged = ctx.run_shell_command("ls .yarn/unplugged").stdout;
    assert_contains!(unplugged, "depd-npm-");
    assert_not_contains!(unplugged, "picocolors-npm-");
    // unplugged packages are loaded from `.yarn/unplugged` so their archives aren't needed at runtime
    let yarn_cache = ctx
        .run_shell_command("ls /layers/heroku_nodejs/yarn_cache")
        .stdout;
    assert_not_contains!(yarn_cache, "depd-npm-");
    assert_not_contains!(yarn_cache, "picocolors-npm-");
    ctx.run_shell_command("node --require ./.pnp.cjs -e \"require('depd')\"");
}

fn assert_yarn_4_pnp_workspace_dependencies(ctx: &TestContext) {
    // the `greeting` workspace depends on `he` (prod) and `picocolors` (dev)
    let yarn_cache = ctx
        .run_shell_command("ls /layers/heroku_nodejs/yarn_cache")
        .stdout;
    assert_contains!(yarn_cache, "he-npm-");
    assert_not_contains!(yarn_cache, "picocolors-npm-");
}

#[derive(Clone, Copy)]
enum Linker {
    Pnp,
    NodeModules,
}

/// Guardrails for how the dependencies of a Yarn 2+ app that doesn't commit its cache are shipped
/// at launch. Every Yarn fixture has `express` as a dependency and `dotenv` as a dev dependency.
fn assert_yarn_berry_runtime(ctx: &TestContext, linker: Linker) {
    // the mirror contains dev dependencies so it must only be available at build time
    let launch_layers = ctx.run_shell_command("ls /layers/heroku_nodejs").stdout;
    assert_not_contains!(launch_layers, "yarn_mirror");

    match linker {
        Linker::Pnp => {
            // dev dependencies are removed from the cache Plug'n'Play loads packages from at launch
            let yarn_cache = ctx
                .run_shell_command("ls /layers/heroku_nodejs/yarn_cache")
                .stdout;
            assert_contains!(yarn_cache, "express-npm-");
            assert_not_contains!(yarn_cache, "dotenv-npm-");
        }
        Linker::NodeModules => {
            let node_modules = ctx.run_shell_command("ls node_modules").stdout;
            assert_contains!(node_modules, "express");
            assert_not_contains!(node_modules, "dotenv");
            // packages are installed into `node_modules` so no package archives are needed at launch
            assert_not_contains!(launch_layers, "yarn_cache");
        }
    }
}

#[test]
#[ignore = "integration test"]
fn yarn_4_modules_zero() {
    nodejs_integration_test_with_config(
        "./fixtures/yarn-4-modules-zero",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                let vendored_yarn = app_dir.join(".yarn/releases/yarn-4.0.0.cjs");
                // Remove executable permissions to test that the buildpack sets them
                let metadata = std::fs::metadata(&vendored_yarn).unwrap();
                let mut permissions = metadata.permissions();
                // Clear user, group, and other executable bits
                let mode = permissions.mode() & !0o111;
                permissions.set_mode(mode);
                std::fs::set_permissions(&vendored_yarn, permissions).unwrap();
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-4-modules-zero");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_yarn_1_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test("./fixtures/yarn-1-project-with-native-module", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_yarn_4_modules_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test("./fixtures/yarn-4-node-modules-with-native-module", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_yarn_4_pnp_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test("./fixtures/yarn-4-pnp-with-native-module", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_yarn_skip_build_scripts_from_buildplan() {
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
fn test_yarn_default_web_process_registration_is_skipped_if_procfile_exists() {
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
fn test_yarn_prune_dev_dependencies_config() {
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
