// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]
// Required due to: https://github.com/rust-lang/rust-clippy/issues/11119
#![allow(clippy::unwrap_used)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{assert_contains, BuildpackReference, PackResult};
use serde_json::json;
use std::path::Path;
use test_support::{
    add_build_script, add_package_json_dependency, create_build_snapshot, custom_buildpack,
    integration_test_with_config, nodejs_integration_test, nodejs_integration_test_with_config,
    update_json_file,
};

#[test]
#[ignore = "integration test"]
fn test_npm_install_with_lockfile() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        create_build_snapshot(&ctx.pack_stdout).assert();
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_install_caching() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
        let config = ctx.config.clone();
        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_install_new_package() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        let build_snapshot = create_build_snapshot(&ctx.pack_stdout);

        let mut config = ctx.config.clone();
        config.app_dir_preprocessor(|app_dir| {
            add_package_json_dependency(&app_dir, "environment", "1.1.0");
            add_lockfile_entry(
                &app_dir,
                "environment",
                json!({
                    "version": "1.1.0",
                    "resolved": "https://registry.npmjs.org/environment/-/environment-1.1.0.tgz",
                    "integrity": "sha512-xUtoPkMggbz0MPyPiIWr1Kp4aeWJjDZ6SMvURhimjdZgsRuDplF5/s9hcgGhyXMhs+6vpnuoiZ2kFiu3FMnS8Q==",
                    "license": "MIT"
                })
            );
        });

        ctx.rebuild(config, |ctx| {
            build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
        });
    });
}

#[test]
#[ignore = "integration test"]
fn test_npm_build_scripts() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
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
    );
}

#[test]
#[ignore = "integration test"]
fn test_npm_build_scripts_prefers_heroku_build_over_build() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                add_build_script(&app_dir, "heroku-build");
                add_build_script(&app_dir, "build");
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_npm_start_script_creates_a_web_process_launcher() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project",
        |config| {
            config.app_dir_preprocessor(|app_dir| {
                add_build_script(&app_dir, "start");
            });
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_dependencies_and_missing_lockfile_errors() {
    nodejs_integration_test_with_config(
        "./fixtures/dependencies-missing-lockfile",
        |cfg| {
            cfg.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            create_build_snapshot(&ctx.pack_stdout).assert();
        },
    );
}

#[test]
#[ignore = "integration test"]
fn detect_rejects_non_npm_project() {
    nodejs_integration_test_with_config(
        "./fixtures/empty",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(ctx.pack_stdout, "fail: heroku/nodejs-npm-install");
        },
    );
}

#[test]
#[ignore = "integration test"]
fn npm_runtime_settings_are_set() {
    nodejs_integration_test("./fixtures/npm-project", |ctx| {
        let env_output = ctx.run_shell_command("env").stdout;
        assert_contains!(env_output, "npm_config_cache=/tmp/npm_cache");
        assert_contains!(env_output, "npm_config_update-notifier=false");
    });
}

#[test]
#[ignore = "integration test"]
fn test_native_modules_are_recompiled_even_on_cache_restore() {
    nodejs_integration_test_with_config(
        "./fixtures/npm-project-with-native-module",
        |config| {
            config.env("npm_config_foreground-scripts", "true");
        },
        |ctx| {
            let build_snapshot = create_build_snapshot(&ctx.pack_stdout);
            let config = ctx.config.clone();
            ctx.rebuild(config, |ctx| {
                build_snapshot.rebuild_output(&ctx.pack_stdout).assert();
            });
        },
    );
}

#[test]
#[ignore = "integration test"]
fn test_skip_build_scripts_from_buildplan() {
    integration_test_with_config(
        "./fixtures/npm-project",
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
        "./fixtures/npm-project",
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
        "./fixtures/npm-project",
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

fn add_lockfile_entry(app_dir: &Path, package_name: &str, lockfile_entry: serde_json::Value) {
    update_json_file(&app_dir.join("package-lock.json"), |json| {
        let packages = json["packages"].as_object_mut().unwrap();
        packages.insert(format!("node_modules/{package_name}"), lockfile_entry);
    });
}
