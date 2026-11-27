// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{BuildpackReference, TestContext, assert_contains, assert_not_contains};
use regex::Regex;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use test_support::{
    add_build_script, assert_web_response, create_build_snapshot, custom_buildpack,
    integration_test_with_config, nodejs_integration_test, nodejs_integration_test_with_config,
    print_build_env_buildpack, update_json_file,
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
        assert_yarn_berry_runtime(&ctx, "yarn-2-pnp-zero", Linker::Pnp, true);
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_2_modules_nonzero() {
    nodejs_integration_test("./fixtures/yarn-2-modules-nonzero", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "yarn-2-modules-nonzero");
        assert_yarn_berry_runtime(&ctx, "yarn-2-modules-nonzero", Linker::NodeModules, false);

        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-2-modules-nonzero");
            assert_yarn_berry_runtime(&ctx, "yarn-2-modules-nonzero", Linker::NodeModules, false);
        });
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_3_pnp_nonzero() {
    nodejs_integration_test("./fixtures/yarn-3-pnp-nonzero", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "yarn-3-pnp-nonzero");
        assert_yarn_berry_runtime(&ctx, "yarn-3-pnp-nonzero", Linker::Pnp, false);

        let mut config = ctx.config.clone();
        config.env(UNREACHABLE_REGISTRY.0, UNREACHABLE_REGISTRY.1);
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-3-pnp-nonzero");
            assert_yarn_berry_runtime(&ctx, "yarn-3-pnp-nonzero", Linker::Pnp, false);

            // the archive of the removed dev dependency is removed from the mirror
            let mut config = ctx.config.clone();
            config.app_dir_preprocessor(|app_dir| remove_dotenv_dependency(&app_dir));
            ctx.rebuild(config, |ctx| {
                assert_contains!(
                    ctx.pack_stdout,
                    "Removed 1 unused package archive from the Yarn mirror"
                );
                assert_contains!(ctx.pack_stdout, "Added 1/1 app layer(s)");
                assert_web_response(&ctx, "yarn-3-pnp-nonzero");
                assert_yarn_berry_runtime(&ctx, "yarn-3-pnp-nonzero", Linker::Pnp, false);
            });
        });
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_3_modules_zero() {
    nodejs_integration_test("./fixtures/yarn-3-modules-zero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-3-modules-zero");
        assert_yarn_berry_runtime(&ctx, "yarn-3-modules-zero", Linker::NodeModules, true);
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_4_pnp_nonzero() {
    nodejs_integration_test("./fixtures/yarn-4-pnp-nonzero", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "yarn-4-pnp-nonzero");
        assert_yarn_berry_runtime(&ctx, "yarn-4-pnp-nonzero", Linker::Pnp, false);
        assert_yarn_4_unplugged_packages(&ctx, "/layers/heroku_nodejs/yarn_cache");

        let mut config = ctx.config.clone();
        config.env(UNREACHABLE_REGISTRY.0, UNREACHABLE_REGISTRY.1);
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-4-pnp-nonzero");
            assert_yarn_berry_runtime(&ctx, "yarn-4-pnp-nonzero", Linker::Pnp, false);
            assert_yarn_4_unplugged_packages(&ctx, "/layers/heroku_nodejs/yarn_cache");

            // the archive of the removed dev dependency is removed from the mirror
            let mut config = ctx.config.clone();
            config.app_dir_preprocessor(|app_dir| remove_dotenv_dependency(&app_dir));
            ctx.rebuild(config, |ctx| {
                assert_contains!(
                    ctx.pack_stdout,
                    "Removed 1 unused package archive from the Yarn mirror"
                );
                assert_contains!(ctx.pack_stdout, "Added 1/1 app layer(s)");
                assert_web_response(&ctx, "yarn-4-pnp-nonzero");
                assert_yarn_berry_runtime(&ctx, "yarn-4-pnp-nonzero", Linker::Pnp, false);
                assert_yarn_4_unplugged_packages(&ctx, "/layers/heroku_nodejs/yarn_cache");
            });
        });
    });
}

#[test]
#[ignore = "integration test"]
fn yarn_4_pnp_zero() {
    nodejs_integration_test("./fixtures/yarn-4-pnp-zero", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
        assert_web_response(&ctx, "yarn-4-pnp-zero");
        assert_yarn_berry_runtime(&ctx, "yarn-4-pnp-zero", Linker::Pnp, true);
        assert_yarn_4_unplugged_packages(&ctx, ".yarn/cache");
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
fn yarn_4_pnp_prune_reruns_postinstall() {
    nodejs_integration_test_with_config(
        "./fixtures/yarn-4-pnp-nonzero",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                update_json_file(&app_dir.join("package.json"), |json| {
                    json["scripts"]["postinstall"] = "node -p \"try { require.resolve('dotenv'), 'with dev dependencies' } catch { 'without dev dependencies' }\" >> postinstall.log".into();
                });
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "yarn-4-pnp-nonzero");
            // Removing dev dependencies changes the app's dependency tree, so Yarn runs its
            // `postinstall` again when pruning, without dev dependencies
            assert_eq!(
                ctx.run_shell_command("cat postinstall.log").stdout,
                "with dev dependencies\nwithout dev dependencies\n"
            );
        },
    );
}

#[test]
#[ignore = "integration test"]
fn yarn_4_pnp_workspaces() {
    nodejs_integration_test("./fixtures/yarn-4-pnp-workspaces", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        assert_web_response(&ctx, "Hello from yarn-4-pnp-workspaces");
        assert_yarn_berry_runtime(&ctx, "yarn-4-pnp-workspaces", Linker::Pnp, false);
        assert_yarn_4_pnp_workspace_dependencies(&ctx);

        let mut config = ctx.config.clone();
        config.env(UNREACHABLE_REGISTRY.0, UNREACHABLE_REGISTRY.1);
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            assert_web_response(&ctx, "Hello from yarn-4-pnp-workspaces");
            assert_yarn_berry_runtime(&ctx, "yarn-4-pnp-workspaces", Linker::Pnp, false);
            assert_yarn_4_pnp_workspace_dependencies(&ctx);
        });
    });
}

/// Removes the `dotenv` dev dependency from the app, as `yarn remove dotenv` would. The buildpack
/// installs with `--immutable`, so the edited lockfile must match what Yarn would write exactly.
/// Deleting text only does that for a package with no dependencies, like `dotenv`, otherwise Yarn
/// would also need to remove the transitive packages it no longer needs which is a much more
/// complicated task.
fn remove_dotenv_dependency(app_dir: &Path) {
    update_json_file(&app_dir.join("package.json"), |json| {
        json["devDependencies"]
            .as_object_mut()
            .expect("package.json should have dev dependencies")
            .remove("dotenv")
            .expect("dotenv should be a dev dependency");
    });
    let lockfile_path = app_dir.join("yarn.lock");
    let mut lockfile =
        std::fs::read_to_string(&lockfile_path).expect("yarn.lock should be readable");
    for pattern in [
        // the package entry
        r#"(?m)^"dotenv@npm:\^16\.5\.0":\n(?: .*\n)*\n"#,
        // the dependency in the workspace entry
        r#"(?m)^    dotenv: "?(?:npm:)?\^16\.5\.0"?\n"#,
    ] {
        let regex = Regex::new(pattern).expect("regex should be valid");
        assert!(
            regex.is_match(&lockfile),
            "{pattern} not found in yarn.lock"
        );
        lockfile = regex.replace(&lockfile, "").into_owned();
    }
    std::fs::write(&lockfile_path, lockfile).expect("yarn.lock should be writable");
}

/// Rebuilds with this registry only pass if every package is restored from the Yarn mirror.
const UNREACHABLE_REGISTRY: (&str, &str) = ("YARN_NPM_REGISTRY_SERVER", "http://127.0.0.1:9");

fn assert_yarn_4_unplugged_packages(ctx: &TestContext, yarn_cache: &str) {
    // `depd` (prod) and `picocolors` (dev) are unplugged with `dependenciesMeta`
    let unplugged = ctx.run_shell_command("ls .yarn/unplugged").stdout;
    assert_contains!(unplugged, "depd-npm-");
    assert_not_contains!(unplugged, "picocolors-npm-");
    // unplugged packages are loaded from `.yarn/unplugged` so their archives aren't needed at runtime
    let yarn_cache = ctx.run_shell_command(format!("ls {yarn_cache}")).stdout;
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

/// Guardrails for how the dependencies of a Yarn 2+ app are shipped at launch. Every Yarn fixture
/// has `express` as a dependency and `dotenv` as a dev dependency.
fn assert_yarn_berry_runtime(ctx: &TestContext, fixture: &str, linker: Linker, zero_install: bool) {
    // the buildpack configures Yarn with build env vars instead of creating or modifying the app's
    // `.yarnrc.yml`
    let fixture_yarnrc = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture)
            .join(".yarnrc.yml"),
    )
    .unwrap_or_default();
    assert_eq!(
        ctx.run_shell_command("cat .yarnrc.yml 2>/dev/null || true")
            .stdout,
        fixture_yarnrc
    );

    // the mirror contains dev dependencies so it must only be available at build time
    let launch_layers = ctx.run_shell_command("ls /layers/heroku_nodejs").stdout;
    assert_not_contains!(launch_layers, "yarn_mirror");

    let zero_install_archives = ctx
        .run_shell_command("find .yarn/cache -name '*.zip' 2>/dev/null || true")
        .stdout;
    match linker {
        Linker::Pnp => {
            // dev dependencies are removed from the cache Plug'n'Play loads packages from at launch
            let archives = if zero_install {
                assert_not_contains!(launch_layers, "yarn_cache");
                zero_install_archives
            } else {
                ctx.run_shell_command("ls /layers/heroku_nodejs/yarn_cache")
                    .stdout
            };
            assert_contains!(archives, "express-npm-");
            assert_not_contains!(archives, "dotenv-npm-");
        }
        Linker::NodeModules => {
            let node_modules = ctx.run_shell_command("ls node_modules").stdout;
            assert_contains!(node_modules, "express");
            assert_not_contains!(node_modules, "dotenv");
            // packages are installed into `node_modules` so no package archives are needed at launch
            assert_not_contains!(launch_layers, "yarn_cache");
            assert_eq!(zero_install_archives, "");
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
            assert_yarn_berry_runtime(&ctx, "yarn-4-modules-zero", Linker::NodeModules, true);
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
