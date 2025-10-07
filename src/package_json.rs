use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::vrs::Requirement;
use bullet_stream::style;
use indoc::formatdoc;
use std::path::{Path, PathBuf};

pub(crate) struct PackageJson(serde_json::Value);

impl PackageJson {
    pub(crate) fn node_engine(&self) -> Option<Requirement> {
        self.0
            .get("engines")
            .and_then(|engines| engines.get("node"))
            .and_then(|node| node.as_str())
            .and_then(|node| Requirement::parse(node).ok())
    }
}

impl TryFrom<PathBuf> for PackageJson {
    type Error = ErrorMessage;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let contents = std::fs::read_to_string(&value)
            .map_err(|e| package_json_read_error_message(&value, &e))?;
        let json = serde_json::from_str::<serde_json::Value>(&contents)
            .map_err(|e| package_json_parse_error_message(&value, &e))?;
        Ok(Self(json))
    }
}

fn package_json_read_error_message(path: &Path, error: &std::io::Error) -> ErrorMessage {
    let package_json = file_value(path);
    error_message()
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error reading {package_json}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {package_json} to complete the build but \
            the file can't be read.

            Suggestions:
            - Ensure the file has read permissions.
        " })
        .debug_info(error.to_string())
        .create()
}

fn package_json_parse_error_message(path: &Path, error: &serde_json::Error) -> ErrorMessage {
    let package_json = file_value(path);
    let json_spec_url = style::url("https://www.json.org/");
    error_message()
        .error_type(ErrorType::UserFacing(
            SuggestRetryBuild::Yes,
            SuggestSubmitIssue::No,
        ))
        .header(format!("Error parsing {package_json}"))
        .body(formatdoc! { "
            The Heroku Node.js buildpack reads from {package_json} to complete the build but \
            the file isn't valid JSON.

            Suggestions:
            - Ensure the file follows the JSON format described at {json_spec_url}
        " })
        .debug_info(error.to_string())
        .create()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bullet_stream::strip_ansi;
    use insta::{assert_snapshot, with_settings};
    use serde_json::json;
    use test_support::test_name;

    #[test]
    fn read_valid_package_with_node_engine() {
        let package_json = PackageJson(json!({
            "engines": {
                "node": "16.0.0"
            }
        }));
        assert_eq!(
            package_json.node_engine().unwrap().to_string(),
            Requirement::parse("16.0.0").unwrap().to_string()
        );
    }

    #[test]
    fn test_read_error_message() {
        assert_error_snapshot(&package_json_read_error_message(
            Path::new("./package.json"),
            &std::io::Error::other("test I/O error blah"),
        ));
    }

    #[test]
    fn test_parse_error_message() {
        assert_error_snapshot(&package_json_parse_error_message(
            Path::new("./package.json"),
            &create_json_error(),
        ));
    }

    fn assert_error_snapshot(error: &ErrorMessage) {
        let error_message = strip_ansi(error.to_string());
        let test_name = format!("package_json__{}", test_name().split("::").last().unwrap());
        with_settings!({
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            assert_snapshot!(test_name, error_message);
        });
    }

    fn create_json_error() -> serde_json::error::Error {
        serde_json::from_str::<serde_json::Value>(r#"{\n  "name":\n}"#).unwrap_err()
    }
}
