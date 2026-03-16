use crate::buildpack_config::{IgnoreEolErrorNodejs, NAMESPACED_CONFIG};
use crate::runtimes::nodejs::NODEJS_VERSIONS;
use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use crate::utils::vrs::Version;
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::formatdoc;
use std::fmt::{Display, Formatter};
use time::Date;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodejsVersionInfo {
    pub(crate) status: NodejsVersionStatus,
    pub(crate) end_of_life_date: Date,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodejsVersionStatus {
    Current,
    ActiveLts,
    MaintenanceLts,
    Eol,
}

impl Display for NodejsVersionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodejsVersionStatus::Current => write!(f, "Current"),
            NodejsVersionStatus::ActiveLts | NodejsVersionStatus::MaintenanceLts => {
                write!(f, "LTS")
            }
            NodejsVersionStatus::Eol => write!(f, "EOL"),
        }
    }
}

const SUPPORT_URL: &str =
    "https://devcenter.heroku.com/articles/nodejs-support#supported-node-js-versions";

/// Checks the support status of a resolved Node.js version against the lifecycle map.
///
/// - If the version is EOL and `ignore_eol_error` is `true`, emits a warning and returns `Ok(())`.
/// - If the version is EOL and `ignore_eol_error` is `false`, returns an error that fails the build.
/// - Returns `Ok(())` for supported versions or unknown versions.
pub(crate) fn check_nodejs_support_status(
    nodejs_version: &Version,
    ignore_eol_error: IgnoreEolErrorNodejs,
) -> Result<(), ErrorMessage> {
    let major_version = nodejs_version.major();
    let Some(version_info) = NODEJS_VERSIONS.get(&major_version) else {
        return Ok(());
    };

    if version_info.status != NodejsVersionStatus::Eol {
        return Ok(());
    }

    if *ignore_eol_error {
        print::warning(create_nodejs_eol_warning(
            major_version,
            version_info.end_of_life_date,
        ));
        Ok(())
    } else {
        Err(create_nodejs_eol_error(
            major_version,
            version_info.end_of_life_date,
        ))
    }
}

fn format_eol_date(date: Date) -> String {
    date.format(
        &time::format_description::parse("[month repr:long] [day padding:none], [year]")
            .expect("format should be valid"),
    )
    .expect("date should format")
}

fn format_supported_versions() -> String {
    let mut supported: Vec<_> = NODEJS_VERSIONS
        .iter()
        .filter(|(_, info)| info.status != NodejsVersionStatus::Eol)
        .collect();
    supported.sort_by_key(|(major, _)| *major);
    supported
        .iter()
        .map(|(major, info)| format!("{major}.x ({})", info.status))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_lts_versions() -> String {
    let mut lts: Vec<_> = NODEJS_VERSIONS
        .iter()
        .filter(|(_, info)| {
            matches!(
                info.status,
                NodejsVersionStatus::ActiveLts | NodejsVersionStatus::MaintenanceLts
            )
        })
        .collect();
    lts.sort_by_key(|(major, _)| *major);
    lts.iter()
        .map(|(major, _)| format!("{major}.x"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn create_nodejs_eol_warning(major_version: u64, eol_date: Date) -> String {
    let eol_date_formatted = format_eol_date(eol_date);
    let supported_versions = format_supported_versions();
    let support_url = style::url(SUPPORT_URL);
    formatdoc! {"
        Node.js {major_version}.x reached end-of-life on {eol_date_formatted} and is no longer \
        supported on Heroku. EOL versions no longer receive security updates or bug fixes from \
        the Node.js project.

        In a future buildpack release, this warning will become a build error. Please upgrade \
        to a supported version as soon as possible to avoid build failures.

        Supported versions: {supported_versions}

        {support_url}
    "}
}

fn create_nodejs_eol_error(major_version: u64, eol_date: Date) -> ErrorMessage {
    let eol_date_formatted = format_eol_date(eol_date);
    let lts_versions = format_lts_versions();
    let project_toml_config = style::value(format!(
        "{NAMESPACED_CONFIG}.support.ignore_eol_error_nodejs = true"
    ));
    let project_toml = style::value("project.toml");
    let support_url = style::url(SUPPORT_URL);
    error_message()
        .id("runtime/nodejs/eol_version")
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::No,
            SuggestSubmitIssue::No,
        ))
        .header("Node.js version not supported")
        .body(formatdoc! {"
            Node.js {major_version}.x reached end-of-life on {eol_date_formatted} and is no longer \
            supported on Heroku.

            Suggestions:
            - Upgrade to a supported LTS version ({lts_versions})
            - Set {project_toml_config} in {project_toml} to temporarily bypass this check

            {support_url}
        "})
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, assert_warning_snapshot};
    use crate::utils::vrs::Version;
    use bullet_stream::global;
    use time::macros::date;

    #[test]
    fn nodejs_eol_warning() {
        assert_warning_snapshot(&create_nodejs_eol_warning(18, date!(2025 - 04 - 30)));
    }

    #[test]
    fn nodejs_eol_error() {
        assert_error_snapshot(&create_nodejs_eol_error(18, date!(2025 - 04 - 30)));
    }

    #[test]
    fn version_not_in_map_returns_ok() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_nodejs_support_status(
                &Version::parse("99.0.0").unwrap(),
                IgnoreEolErrorNodejs::from(false),
            );
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(!output.contains("end-of-life"));
    }

    #[test]
    fn current_version_returns_ok() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_nodejs_support_status(
                &Version::parse("25.0.0").unwrap(),
                IgnoreEolErrorNodejs::from(false),
            );
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(!output.contains("end-of-life"));
    }

    #[test]
    fn active_lts_version_returns_ok() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_nodejs_support_status(
                &Version::parse("24.0.0").unwrap(),
                IgnoreEolErrorNodejs::from(false),
            );
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(!output.contains("end-of-life"));
    }

    #[test]
    fn maintenance_lts_version_returns_ok() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_nodejs_support_status(
                &Version::parse("22.0.0").unwrap(),
                IgnoreEolErrorNodejs::from(false),
            );
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(!output.contains("end-of-life"));
    }

    #[test]
    fn eol_version_with_ignore_emits_warning_returns_ok() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_nodejs_support_status(
                &Version::parse("18.0.0").unwrap(),
                IgnoreEolErrorNodejs::from(true),
            );
            assert!(result.is_ok());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(output.contains("end-of-life"));
    }

    #[test]
    fn eol_version_without_ignore_returns_err_without_warning() {
        let log = global::with_locked_writer(Vec::<u8>::new(), || {
            let result = check_nodejs_support_status(
                &Version::parse("18.0.0").unwrap(),
                IgnoreEolErrorNodejs::from(false),
            );
            assert!(result.is_err());
        });
        let output = String::from_utf8_lossy(&log);
        assert!(
            !output.contains("end-of-life"),
            "warning should not be emitted when error is returned"
        );
    }
}
