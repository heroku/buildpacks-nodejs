use crate::error_handling::ErrorType::UserFacing;
use crate::package_json::PackageJsonError;
use bullet_stream::{style, Print};
use indoc::formatdoc;
use std::fmt::{Display, Formatter};
use std::path::Path;

const BUILDPACK_NAME: &str = "Heroku Node.js Buildpack";

const ISSUES_URL: &str = "https://github.com/heroku/buildpacks-nodejs/issues";

pub fn on_framework_error<E>(error: &E) -> ErrorMessage
where
    E: Display,
{
    let issues_url = style::url(ISSUES_URL);
    error_message()
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

#[must_use]
pub fn on_package_json_error(error: PackageJsonError) -> ErrorMessage {
    let package_json = file_value("./package.json");
    let json_spec_url = style::url("https://www.json.org/");
    match error {
        PackageJsonError::AccessError(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
            .header(format!("Error reading {package_json}"))
            .body(formatdoc! { "
                The {BUILDPACK_NAME} reads from {package_json} to complete the build but \
                the file can't be read.

                Suggestions:
                - Ensure the file has read permissions.
            " })
            .debug_info(e.to_string())
            .create(),

        PackageJsonError::ParseError(e) => error_message()
            .error_type(UserFacing(SuggestRetryBuild::Yes, SuggestSubmitIssue::No))
            .header(format!("Error parsing {package_json}"))
            .body(formatdoc! { "
                The {BUILDPACK_NAME} reads from {package_json} to complete the build but \
                the file isn't valid JSON.

                Suggestions:
                - Ensure the file follows the JSON format described at {json_spec_url}
            " })
            .debug_info(e.to_string())
            .create(),
    }
}

#[bon::builder(finish_fn = create, on(String, into), state_mod(vis = "pub"))]
#[allow(clippy::needless_pass_by_value)]
pub fn error_message(
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
    }
}

pub fn file_value(value: impl AsRef<Path>) -> String {
    style::value(value.as_ref().to_string_lossy())
}

#[derive(Debug)]
pub struct ErrorMessage {
    debug_info: Option<String>,
    message: String,
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

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    Framework,
    Internal,
    UserFacing(SuggestRetryBuild, SuggestSubmitIssue),
}

#[derive(Debug, PartialEq)]
pub enum SuggestRetryBuild {
    Yes,
    No,
}

#[derive(Debug, PartialEq)]
pub enum SuggestSubmitIssue {
    Yes,
    No,
}
