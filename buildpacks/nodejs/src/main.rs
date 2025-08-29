use crate::corepack::main::CorepackBuildpackError;
use crate::engine::main::NodeJsEngineBuildpackError;
use crate::npm_engine::main::NpmEngineBuildpackError;
use crate::npm_install::main::NpmInstallBuildpackError;
use crate::pnpm_engine::main::PnpmEngineBuildpackError;
use crate::pnpm_install::main::PnpmInstallBuildpackError;
use crate::yarn::main::YarnBuildpackError;
use bullet_stream::global::print;
use heroku_nodejs_utils::error_handling::on_framework_error;
use libcnb::build::BuildResultBuilder;
use libcnb::data::build_plan::BuildPlanBuilder;
use libcnb::detect::DetectResultBuilder;
use libcnb::{buildpack_main, Env};
#[cfg(test)]
use libcnb_test as _;
#[cfg(test)]
use regex as _;

mod corepack;
mod engine;
mod npm_engine;
mod npm_install;
mod pnpm_engine;
mod pnpm_install;
mod yarn;

type BuildpackDetectContext = libcnb::detect::DetectContext<NodeJsBuildpack>;
type BuildpackBuildContext = libcnb::build::BuildContext<NodeJsBuildpack>;
type BuildpackError = libcnb::Error<NodeJsBuildpackError>;
type BuildpackResult<T> = Result<T, BuildpackError>;

buildpack_main!(NodeJsBuildpack);

struct NodeJsBuildpack;

impl libcnb::Buildpack for NodeJsBuildpack {
    type Platform = libcnb::generic::GenericPlatform;
    type Metadata = libcnb::generic::GenericMetadata;
    type Error = NodeJsBuildpackError;

    fn detect(
        &self,
        context: BuildpackDetectContext,
    ) -> libcnb::Result<libcnb::detect::DetectResult, NodeJsBuildpackError> {
        let buildpack_id = context.buildpack_descriptor.buildpack.id.to_string();

        // provide heroku/nodejs for other buildpacks to use
        let mut buildplan_builder = BuildPlanBuilder::new().provides(&buildpack_id);

        // If there are common node artifacts, this buildpack should both
        // provide and require heroku/nodejs so that it may be used as
        // a standalone buildpack
        if ["package.json", "server.js", "index.js"]
            .map(|name| context.app_dir.join(name))
            .iter()
            .any(|path| path.exists())
        {
            buildplan_builder = buildplan_builder.requires(&buildpack_id);
        }

        DetectResultBuilder::pass()
            .build_plan(buildplan_builder.build())
            .build()
    }

    fn build(
        &self,
        context: BuildpackBuildContext,
    ) -> libcnb::Result<libcnb::build::BuildResult, NodeJsBuildpackError> {
        let buildpack_start = print::buildpack(
            context
                .buildpack_descriptor
                .buildpack
                .name
                .as_ref()
                .expect("The buildpack should have a name"),
        );

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
            } else if npm_install::main::detect(&context)? {
                (_, build_result_builder) =
                    npm_install::main::build(&context, env, build_result_builder)?;
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

        print::all_done(&Some(buildpack_start));

        build_result_builder.build()
    }

    fn on_error(&self, error: BuildpackError) {
        let error_message = match error {
            libcnb::Error::BuildpackError(buildpack_error) => match buildpack_error {
                NodeJsBuildpackError::NodeEngine(error) => engine::main::on_error(error),
                NodeJsBuildpackError::Corepack(error) => corepack::main::on_error(error),
                NodeJsBuildpackError::NpmEngine(error) => npm_engine::main::on_error(error),
                NodeJsBuildpackError::NpmInstall(error) => npm_install::main::on_error(error),
                NodeJsBuildpackError::PnpmInstall(error) => pnpm_install::main::on_error(error),
                NodeJsBuildpackError::PnpmEngine(error) => pnpm_engine::main::on_error(error),
                NodeJsBuildpackError::Yarn(error) => yarn::main::on_error(error),
            },
            framework_error => on_framework_error(&framework_error),
        };
        print::plain(error_message.to_string());
        eprintln!();
    }
}

#[derive(Debug)]
enum NodeJsBuildpackError {
    NodeEngine(NodeJsEngineBuildpackError),
    Corepack(CorepackBuildpackError),
    NpmEngine(NpmEngineBuildpackError),
    NpmInstall(NpmInstallBuildpackError),
    PnpmInstall(PnpmInstallBuildpackError),
    PnpmEngine(PnpmEngineBuildpackError),
    Yarn(YarnBuildpackError),
}

impl From<NodeJsBuildpackError> for BuildpackError {
    fn from(value: NodeJsBuildpackError) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}
