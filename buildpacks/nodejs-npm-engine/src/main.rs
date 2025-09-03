use bullet_stream::global::print;
use bullet_stream::style;
use indoc::formatdoc;
use libcnb::build::{BuildContext, BuildResult};
use libcnb::detect::{DetectContext, DetectResult, DetectResultBuilder};
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

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let buildpack_id = style::value(context.buildpack_descriptor.buildpack.id.to_string());
        let replacement_buildpack_id = style::value("heroku/nodejs");
        let project_toml = style::value("project.toml");
        print::error(formatdoc! { "
            Usage of {buildpack_id} is deprecated and will no longer be supported beyond v4.1.4.

            Equivalent functionality is now provided by the {replacement_buildpack_id} buildpack:
            - Buildpacks authors that previously required {buildpack_id} should now require {replacement_buildpack_id} instead.
            - Users with a {project_toml} file that lists {buildpack_id} should now use {replacement_buildpack_id} instead.

            If you have any questions, please file an issue at https://github.com/heroku/buildpacks-nodejs/issues/new.
        " });
        DetectResultBuilder::fail().build()
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        unimplemented!("This will never run since detect is configured to always fail.");
    }
}
