use crate::corepack::main::CorepackBuildpackError;
use crate::engine::main::NodeJsEngineBuildpackError;
use crate::npm_engine::main::NpmEngineBuildpackError;
use crate::npm_install::main::NpmInstallBuildpackError;
use crate::pnpm_engine::main::PnpmEngineBuildpackError;
use crate::pnpm_install::main::PnpmInstallBuildpackError;
use crate::yarn::main::YarnBuildpackError;
use heroku_nodejs_utils::error_handling::on_framework_error;

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

struct NodeJsBuildpack;

impl libcnb::Buildpack for NodeJsBuildpack {
    type Platform = libcnb::generic::GenericPlatform;
    type Metadata = libcnb::generic::GenericMetadata;
    type Error = NodeJsBuildpackError;

    fn detect(
        &self,
        context: BuildpackDetectContext,
    ) -> libcnb::Result<libcnb::detect::DetectResult, NodeJsBuildpackError> {
        todo!()
    }

    fn build(
        &self,
        context: BuildpackBuildContext,
    ) -> libcnb::Result<libcnb::build::BuildResult, NodeJsBuildpackError> {
        todo!()
    }

    fn on_error(&self, error: BuildpackError) {
        let error_message = match error {
            libcnb::Error::BuildpackError(buildpack_error) => match buildpack_error {
                NodeJsBuildpackError::NodeEngine(error) => {
                    engine::main::on_error(error);
                }
                NodeJsBuildpackError::Corepack(error) => {
                    corepack::main::on_error(error);
                }
                NodeJsBuildpackError::NpmEngine(error) => {
                    npm_engine::main::on_error(error);
                }
                NodeJsBuildpackError::NpmInstall(error) => {
                    npm_install::main::on_error(error);
                }
                NodeJsBuildpackError::PnpmInstall(error) => {
                    pnpm_install::main::on_error(error);
                }
                NodeJsBuildpackError::PnpmEngine(error) => {
                    pnpm_engine::main::on_error(error);
                }
                NodeJsBuildpackError::Yarn(error) => {
                    yarn::main::on_error(error);
                }
            },
            framework_error => on_framework_error(&framework_error),
        };
        bullet_stream::global::print::error(error_message.to_string());
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
