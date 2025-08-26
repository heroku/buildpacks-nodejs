use libcnb::build::{BuildContext, BuildResult};
use libcnb::detect::{DetectContext, DetectResult};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::Buildpack;

struct DeprecatedBuildpack;
struct DeprecatedBuildpackError;

impl Buildpack for DeprecatedBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = DeprecatedBuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        todo!("should deprecation message be handled here or in build?")
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        todo!("should deprecation message be handled here or in detect?")
    }
}
