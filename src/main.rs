use crate::common::buildplan::NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME;
use crate::corepack::main::CorepackBuildpackError;
use crate::engine::main::NodeJsEngineBuildpackError;
use crate::npm_engine::main::NpmEngineBuildpackError;
use crate::npm_install::main::NpmInstallBuildpackError;
use crate::pnpm_engine::main::PnpmEngineBuildpackError;
use crate::pnpm_install::main::PnpmInstallBuildpackError;
use crate::yarn::main::YarnBuildpackError;
use libcnb::build::{BuildContext, BuildResult, BuildResultBuilder};
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::Error::BuildpackError;
use libcnb::{buildpack_main, Buildpack, Env, Error};

mod common;
mod corepack;
mod engine;
mod npm_engine;
mod npm_install;
mod pnpm_engine;
mod pnpm_install;
mod yarn;

buildpack_main!(NodejsBuildpack);

struct NodejsBuildpack;

impl Buildpack for NodejsBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = NodejsBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // If there are common node artifacts, this buildpack should both
        // provide and require node so that it may be used without other
        // buildpacks.
        if ["package.json", "server.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .any(|path| path.exists())
        {
            DetectResultBuilder::pass()
                .build_plan(
                    BuildPlanBuilder::new()
                        .provides(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                        .requires(NODE_BUILD_SCRIPTS_BUILD_PLAN_NAME)
                        .build(),
                )
                .build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let mut build_result_builder = BuildResultBuilder::new();
        let mut env = Env::from_current();

        (env, build_result_builder) = engine::main::build(&context, env, build_result_builder)?;

        if corepack::main::detect(&context)? {
            (env, build_result_builder) =
                corepack::main::build(&context, env, build_result_builder)?;
            if pnpm_install::main::detect(&context)? {
                (_, build_result_builder) =
                    pnpm_install::main::build(&context, env, build_result_builder)?;
            } else if yarn::main::detect(&context)? {
                (_, build_result_builder) = yarn::main::build(&context, env, build_result_builder)?;
            }
        } else if npm_install::main::detect(&context)? {
            if npm_engine::main::detect(&context)? {
                (env, build_result_builder) =
                    npm_engine::main::build(&context, env, build_result_builder)?;
            }
            (_, build_result_builder) =
                npm_install::main::build(&context, env, build_result_builder)?;
        } else if pnpm_install::main::detect(&context)? {
            if pnpm_engine::main::detect(&context)? {
                (env, build_result_builder) =
                    pnpm_engine::main::build(&context, env, build_result_builder)?;
            }
            (_, build_result_builder) =
                pnpm_install::main::build(&context, env, build_result_builder)?;
        } else if yarn::main::detect(&context)? {
            (_, build_result_builder) = yarn::main::build(&context, env, build_result_builder)?;
        }

        build_result_builder.build()
    }

    fn on_error(&self, error: Error<Self::Error>) {
        match error {
            BuildpackError(error) => match error {
                NodejsBuildpackError::NodeEngine(error) => {
                    engine::main::on_error(error);
                }
                NodejsBuildpackError::Corepack(error) => {
                    corepack::main::on_error(error);
                }
                NodejsBuildpackError::NpmEngine(error) => {
                    npm_engine::main::on_error(error);
                }
                NodejsBuildpackError::NpmInstall(error) => {
                    npm_install::main::on_error(error);
                }
                NodejsBuildpackError::PnpmInstall(error) => {
                    pnpm_install::main::on_error(error);
                }
                NodejsBuildpackError::PnpmEngine(error) => {
                    pnpm_engine::main::on_error(error);
                }
                NodejsBuildpackError::Yarn(error) => {
                    yarn::main::on_error(error);
                }
            },
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
enum NodejsBuildpackError {
    NodeEngine(NodeJsEngineBuildpackError),
    Corepack(CorepackBuildpackError),
    NpmEngine(NpmEngineBuildpackError),
    NpmInstall(NpmInstallBuildpackError),
    PnpmInstall(PnpmInstallBuildpackError),
    PnpmEngine(PnpmEngineBuildpackError),
    Yarn(YarnBuildpackError),
}
