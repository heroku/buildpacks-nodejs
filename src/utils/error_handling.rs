use crate::BuildpackError;
use bullet_stream::{Print, style};
use indoc::formatdoc;
use std::fmt::{Display, Formatter};
use std::path::Path;

pub(crate) const BUILDPACK_NAME: &str = "Heroku Node.js buildpack";

const ISSUES_URL: &str = "https://github.com/heroku/buildpacks-nodejs/issues";

pub(crate) fn on_framework_error<E>(error: &E) -> ErrorMessage
where
    E: Display,
{
    let issues_url = style::url(ISSUES_URL);
    error_message()
        .id("framework_error")
        .error_type(ErrorType::Framework)
        .header(format!("{BUILDPACK_NAME} internal error"))
        .body(formatdoc! {"
            The framework used by this buildpack encountered an unexpected error.

            If you can’t deploy to Heroku due to this issue, check the official Heroku Status page at \
            status.heroku.com for any ongoing incidents. After all incidents resolve, retry your build.

            Use the debug information above to troubleshoot and retry your build. If you think you found a \
            bug in the buildpack, reproduce the issue locally with a minimal example and file an issue here:
            {issues_url}
        "})
        .debug_info(error.to_string())
        .create()
}

#[bon::builder(finish_fn = create, on(String, into), state_mod(vis = "pub"))]
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn error_message(
    id: String,
    header: String,
    body: String,
    error_type: ErrorType,
    debug_info: Option<String>,
) -> ErrorMessage {
    let mut message_parts = vec![header.trim().to_string(), body.trim().to_string()];
    let issues_url = style::url(ISSUES_URL);
    let pack = style::value("pack");
    let pack_url =
        style::url("https://buildpacks.io/docs/for-platform-operators/how-to/integrate-ci/pack/");

    match error_type {
        ErrorType::Framework => {}
        ErrorType::Internal => {
            message_parts.push(formatdoc! { "
                The causes for this error are unknown. We do not have suggestions for diagnosis or a \
                workaround at this time. You can help our understanding by sharing your buildpack log \
                and a description of the issue at:
                {issues_url}

                If you're able to reproduce the problem with an example application and the {pack} \
                build tool ({pack_url}), adding that information to the discussion will also help. Once \
                we have more information around the causes of this error we may update this message.
            "});
        }
        ErrorType::UserFacing(suggest_retry_build, suggest_submit_issue) => {
            if let SuggestRetryBuild::Yes = suggest_retry_build {
                message_parts.push(
                    formatdoc! { "
                        Use the debug information above to troubleshoot and retry your build.
                    "}
                    .trim()
                    .to_string(),
                );
            }

            if let SuggestSubmitIssue::Yes = suggest_submit_issue {
                message_parts.push(formatdoc! { "
                    If the issue persists and you think you found a bug in the buildpack, reproduce \
                    the issue locally with a minimal example. Open an issue in the buildpack's GitHub \
                    repository and include the details here:
                    {issues_url}
                "}.trim().to_string());
            }
        }
    }

    let message = message_parts.join("\n\n");

    ErrorMessage {
        debug_info,
        message,
        id,
    }
}

pub(crate) fn file_value(value: impl AsRef<Path>) -> String {
    style::value(value.as_ref().to_string_lossy())
}

#[derive(Debug)]
pub(crate) struct ErrorMessage {
    debug_info: Option<String>,
    message: String,
    pub(crate) id: String,
}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut log = Print::new(vec![]).without_header();
        if let Some(debug_info) = &self.debug_info {
            log = log
                .bullet(style::important("Debug Info:"))
                .sub_bullet(debug_info)
                .done();
        }
        let output = log.error(&self.message);
        write!(f, "{}", String::from_utf8_lossy(&output))
    }
}

impl From<ErrorMessage> for BuildpackError {
    fn from(value: ErrorMessage) -> Self {
        libcnb::Error::BuildpackError(value)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ErrorType {
    Framework,
    Internal,
    UserFacing(SuggestRetryBuild, SuggestSubmitIssue),
}

#[derive(Debug, PartialEq)]
pub(crate) enum SuggestRetryBuild {
    Yes,
    No,
}

#[derive(Debug, PartialEq)]
pub(crate) enum SuggestSubmitIssue {
    Yes,
    No,
}

#[cfg(test)]
pub(crate) mod test_util {
    use crate::utils::error_handling::ErrorMessage;
    use bullet_stream::strip_ansi;
    use fun_run::{CmdError, CommandWithName};
    use insta::{assert_snapshot, with_settings};
    use std::path::PathBuf;
    use std::process::Command;
    use test_support::test_name;

    pub(crate) fn assert_error_snapshot(error: &ErrorMessage) {
        let error_message = strip_ansi(error.to_string());
        let test_name = test_name()
            .replace("::", "_")
            .replace("_tests", "")
            .replace("_test", "");
        let snapshot_path = std::env::var("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .expect(
                "The CARGO_MANIFEST_DIR should be automatically set by Cargo when running tests",
            )
            .join("src/__snapshots");
        with_settings!({
            prepend_module_to_snapshot => false,
            omit_expression => true,
            snapshot_path => snapshot_path,
        }, {
            assert_snapshot!(test_name, error_message);
        });
    }

    pub(crate) fn create_reqwest_error() -> reqwest_middleware::Error {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            reqwest_middleware::Error::Reqwest(
                reqwest::get("https://test/error").await.unwrap_err(),
            )
        })
    }

    pub(crate) fn create_json_error() -> serde_json::error::Error {
        serde_json::from_str::<serde_json::Value>(r#"{\n  "name":\n}"#).unwrap_err()
    }

    pub(crate) fn create_cmd_error(command: impl Into<String>) -> CmdError {
        Command::new("false")
            .named(command.into())
            .named_output()
            .unwrap_err()
    }
}
