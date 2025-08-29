// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use indoc::indoc;
use libcnb::data::buildpack_id;
use libcnb_test::{assert_contains, BuildpackReference, PackResult};
use test_support::{custom_buildpack, integration_test_with_config};

#[test]
#[ignore = "integration test"]
fn test_nodejs_engine_is_deprecated() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-engine` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-engine` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-engine` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-engine"
        ))],
    );
}

#[test]
#[ignore = "integration test"]
fn test_nodejs_corepack_is_deprecated() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-corepack` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-corepack` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-corepack` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-corepack"
        ))],
    );
}

#[test]
#[ignore = "integration test"]
fn test_nodejs_npm_engine_is_deprecated() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-npm-engine` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-npm-engine` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-npm-engine` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-npm-engine"
        ))],
    );
}

#[test]
#[ignore = "integration test"]
fn test_nodejs_npm_install_is_deprecated() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-npm-install` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-npm-install` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-npm-install` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-npm-install"
        ))],
    );
}

#[test]
#[ignore = "integration test"]
fn test_nodejs_pnpm_engine_is_deprecated() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-pnpm-engine` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-pnpm-engine` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-pnpm-engine` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-pnpm-engine"
        ))],
    );
}

#[test]
#[ignore = "integration test"]
fn test_nodejs_pnpm_install_is_deprecated() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-pnpm-install` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-pnpm-install` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-pnpm-install` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-pnpm-install"
        ))],
    );
}

#[test]
#[ignore = "integration test"]
fn test_nodejs_yarn_is_deprecated() {
    integration_test_with_config(
        "./fixtures/node-with-indexjs",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-yarn` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-yarn` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-yarn` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[BuildpackReference::WorkspaceBuildpack(buildpack_id!(
            "heroku/nodejs-yarn"
        ))],
    );
}

#[test]
#[ignore = "integration test"]
fn test_deprecated_buildpacks_required_in_build_plan() {
    integration_test_with_config(
        "./fixtures/yarn-project",
        |config| {
            config.expected_pack_result(PackResult::Failure);
        },
        |ctx| {
            assert_contains!(
                ctx.pack_stdout,
                indoc! { "
                    ! Usage of `heroku/nodejs-engine` is deprecated and will no longer be supported beyond v4.1.3.
                    !
                    ! Equivalent functionality is now provided by the `heroku/nodejs` buildpack:
                    ! - Buildpacks authors that previously required `heroku/nodejs-engine` should now require `heroku/nodejs` instead.
                    ! - Users with a `project.toml` file that lists `heroku/nodejs-engine` should now use `heroku/nodejs` instead.
                    !
                    ! If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
                "}
            );
        },
        &[
            BuildpackReference::WorkspaceBuildpack(buildpack_id!("heroku/nodejs-engine")),
            BuildpackReference::Other(
                custom_buildpack()
                    .id("test/require-nodejs")
                    .detect(indoc! { r#"
                        #!/usr/bin/env bash

                        build_plan="$2"

                        cat <<EOF >"$build_plan"
                            [[requires]]
                            name = "heroku/nodejs-engine"
                        EOF
                    "# })
                    .call(),
            ),
        ],
    );
}
