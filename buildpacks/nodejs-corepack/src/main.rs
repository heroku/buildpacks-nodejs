use libcnb::build::{BuildContext, BuildResult};
use libcnb::detect::{DetectContext, DetectResult};
use libcnb::generic::{GenericMetadata, GenericPlatform};
use libcnb::{buildpack_main, Buildpack};

buildpack_main!(DeprecatedBuildpack);

struct DeprecatedBuildpack;

#[derive(Debug)]
struct DeprecatedBuildpackError;

impl Buildpack for DeprecatedBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = DeprecatedBuildpackError;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        todo!("should deprecation message be handled here or in build?")
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        todo!("should deprecation message be handled here or in detect?")
    }
}
