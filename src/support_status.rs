use crate::BuildpackResult;
use crate::o11y::RUNTIME_SUPPORT_STATUS;
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::formatdoc;
use nodejs_data::{NodejsArtifact, SUPPORTED_NODEJS_VERSIONS};
use tracing::instrument;

#[instrument(skip_all)]
pub(crate) fn check_nodejs_support_status(artifact: &NodejsArtifact) -> BuildpackResult<()> {
    if SUPPORTED_NODEJS_VERSIONS.contains(&artifact.version.major()) {
        tracing::info!({ RUNTIME_SUPPORT_STATUS } = "supported", "support_status");
        Ok(())
    } else {
        tracing::info!({ RUNTIME_SUPPORT_STATUS } = "eol_warning", "support_status");
        print::warning(create_eol_warning(&artifact.version));
        Ok(())
    }
}

fn create_eol_warning(version: &nodejs_data::Version) -> String {
    let version = style::value(version.to_string());
    let support_url = style::url(
        "https://devcenter.heroku.com/articles/nodejs-support#supported-node-js-versions",
    );
    formatdoc! {"
        Node.js {version} is now End-of-Life (EOL). It no longer receives security \
        updates, bug fixes, or support from the Node.js project and is no longer supported on Heroku.

        In a future buildpack release, this warning will become a build error. Please upgrade to a supported \
        version as soon as possible to avoid build failures.

        {support_url}
    "}
}
