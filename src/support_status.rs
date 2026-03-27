use crate::BuildpackResult;
use crate::o11y::*;
use crate::runtimes::nodejs::NODEJS_INVENTORY;
use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::formatdoc;
use nodejs_data::NodejsArtifact;
use tracing::instrument;

const NODEJS_VERSION_SUPPORT_URL: &str =
    "https://devcenter.heroku.com/articles/nodejs-support#supported-node-js-versions";

#[instrument(skip_all)]
pub(crate) fn check_nodejs_support_status(artifact: &NodejsArtifact) -> BuildpackResult<()> {
    let today = time::OffsetDateTime::now_utc().date();
    let version = &artifact.version;
    let eol_date = nodejs_data::eol_date_for_version(version, &NODEJS_INVENTORY);

    if today < eol_date {
        tracing::info!({ RUNTIME_SUPPORT_STATUS } = "supported", "support_status");
        Ok(())
    } else if nodejs_data::REJECTED_VERSIONS.contains(&version.major()) {
        tracing::info!({ RUNTIME_SUPPORT_STATUS } = "eol_error", "support_status");
        Err(create_eol_error(version, eol_date).into())
    } else {
        tracing::info!({ RUNTIME_SUPPORT_STATUS } = "eol_warning", "support_status");
        print::warning(create_eol_warning(version, eol_date));
        Ok(())
    }
}

fn format_lts_versions() -> String {
    let mut versions = nodejs_data::maintenance_lts_versions(&NODEJS_INVENTORY.schedule);
    versions.push(nodejs_data::active_lts_version(&NODEJS_INVENTORY.schedule));
    versions.sort_unstable();
    versions
        .iter()
        .map(|v| format!("{v}.x"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn create_eol_warning(version: &nodejs_data::Version, eol_date: time::Date) -> String {
    let version = style::value(match (version.major(), version.minor()) {
        (0, minor) => format!("v0.{minor}"),
        (major, _) => format!("v{major}"),
    });
    let eol = style::value(eol_date.to_string());
    let lts_versions = format_lts_versions();
    let support_url = style::url(NODEJS_VERSION_SUPPORT_URL);
    formatdoc! {"
        Node.js {version} reached its official End-of-Life (EOL) on {eol}. It no longer receives security \
        updates, bug fixes, or support from the Node.js project and is no longer supported on Heroku.

        In a future buildpack release, this warning will become a build error. Please upgrade to a supported \
        version as soon as possible to avoid build failures.

        Supported versions: {lts_versions}

        {support_url}
    "}
}

fn create_eol_error(version: &nodejs_data::Version, eol_date: time::Date) -> ErrorMessage {
    let version = style::value(match (version.major(), version.minor()) {
        (0, minor) => format!("v0.{minor}"),
        (major, _) => format!("v{major}"),
    });
    let eol = style::value(eol_date.to_string());
    let support_url = style::url(NODEJS_VERSION_SUPPORT_URL);
    let lts_versions = format_lts_versions();
    error_message()
        .id("runtime/nodejs/eol_version")
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::No))
        .header(format!("Support for Node.js {version} has ended"))
        .body(formatdoc! {"
            Node.js {version} reached its official End-of-Life (EOL) on {eol}. It no longer receives security \
            updates, bug fixes, or support from the Node.js project and is no longer supported on Heroku.

            Suggestions:
            - Upgrade to a supported LTS version ({lts_versions})

            {support_url}
        "})
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, assert_warning_snapshot};

    #[test]
    fn eol_warning_message() {
        let version = nodejs_data::Version::parse("18.0.0").expect("version should be valid");
        let eol_date = time::Date::from_calendar_date(2025, time::Month::April, 30)
            .expect("date should be valid");
        assert_warning_snapshot(&create_eol_warning(&version, eol_date));
    }

    #[test]
    fn eol_error_message() {
        let version = nodejs_data::Version::parse("18.0.0").expect("version should be valid");
        let eol_date = time::Date::from_calendar_date(2025, time::Month::April, 30)
            .expect("date should be valid");
        assert_error_snapshot(&create_eol_error(&version, eol_date));
    }
}
