use crate::BuildpackResult;
use crate::o11y::{PACKAGE_MANAGER_NPM_SUPPORT_STATUS, RUNTIME_SUPPORT_STATUS};
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::formatdoc;
use nodejs_data::{
    MINIMUM_SUPPORTED_NPM_VERSION, NodejsArtifact, SUPPORTED_NODEJS_VERSIONS, Version,
};
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

#[instrument(skip_all)]
pub(crate) fn check_npm_support_status(npm_version: &Version) -> BuildpackResult<()> {
    if npm_version >= &*MINIMUM_SUPPORTED_NPM_VERSION {
        tracing::info!(
            { PACKAGE_MANAGER_NPM_SUPPORT_STATUS } = "supported",
            "package_manager"
        );
        Ok(())
    } else {
        tracing::info!(
            { PACKAGE_MANAGER_NPM_SUPPORT_STATUS } = "unsupported_warning",
            "package_manager"
        );
        print::warning(create_npm_unsupported_warning(npm_version));
        Ok(())
    }
}

fn create_npm_unsupported_warning(npm_version: &Version) -> String {
    let npm_version = style::value(npm_version.to_string());
    let minimum_version = style::value(MINIMUM_SUPPORTED_NPM_VERSION.to_string());
    let support_url = style::url(NPM_SUPPORT_URL);
    formatdoc! {"
        npm {npm_version} is no longer supported on Heroku. The npm maintainers only \
        make regular releases to the most recent major release-line and recommend \
        using the latest version of npm.

        Please upgrade to npm {minimum_version} or later to avoid build failures in a \
        future buildpack release.

        {support_url}
    "}
}

const NPM_SUPPORT_URL: &str =
    "https://devcenter.heroku.com/articles/nodejs-support#npm-version-policy";

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::global;
    use bullet_stream::strip_ansi;
    use insta::{assert_snapshot, with_settings};
    use std::path::PathBuf;
    use test_support::test_name;

    fn assert_warning_snapshot(warning: &str) {
        let warning = strip_ansi(warning);
        let test_name = test_name()
            .replace("::", "_")
            .replace("_tests", "")
            .replace("_test", "");
        let snapshot_path = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect("CARGO_MANIFEST_DIR should be set by Cargo")
            .join("src/__snapshots");
        with_settings!({
            prepend_module_to_snapshot => false,
            omit_expression => true,
            snapshot_path => snapshot_path,
        }, {
            assert_snapshot!(test_name, warning);
        });
    }

    #[test]
    fn npm_unsupported_warning() {
        assert_warning_snapshot(&create_npm_unsupported_warning(&Version::new(8, 19, 4)));
    }

    #[test]
    fn npm_version_below_minimum_emits_warning() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_npm_support_status(&Version::new(8, 19, 4));
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(
            output.contains("no longer supported"),
            "expected warning in output, got: {output}"
        );
    }

    #[test]
    fn npm_version_at_minimum_no_warning() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_npm_support_status(&Version::new(9, 6, 4));
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(
            !output.contains("no longer supported"),
            "unexpected warning in output: {output}"
        );
    }

    #[test]
    fn npm_version_above_minimum_no_warning() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_npm_support_status(&Version::new(11, 0, 0));
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(
            !output.contains("no longer supported"),
            "unexpected warning in output: {output}"
        );
    }
}
