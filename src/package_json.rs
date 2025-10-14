use crate::utils::error_handling::{
    ErrorMessage, ErrorType, SuggestRetryBuild, SuggestSubmitIssue, error_message, file_value,
};
use crate::utils::vrs::{Requirement, Version, VersionError};
use bullet_stream::style;
use indoc::formatdoc;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub(crate) struct PackageJson(serde_json::Value);

impl PackageJson {
    pub(crate) fn node_engine(&self) -> Option<Result<Requirement, VersionError>> {
        self.engines()
            .and_then(|engines| engines.get("node"))
            .and_then(|node| node.as_str())
            .map(Requirement::parse)
    }

    pub(crate) fn npm_engine(&self) -> Option<Requirement> {
        self.engines()
            .and_then(|engines| engines.get("npm"))
            .and_then(|node| node.as_str())
            .and_then(|node| Requirement::parse(node).ok())
    }

    fn engines(&self) -> Option<&serde_json::Value> {
        self.0.get("engines")
    }

    pub(crate) fn package_manager(
        &self,
    ) -> Option<Result<PackageManagerField, PackageManagerFieldError>> {
        self.0
            .get("packageManager")
            .and_then(|val| val.as_str())
            .map(PackageManagerField::from_str)
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

#[derive(Debug, PartialEq)]
pub(crate) struct PackageManagerField {
    pub(crate) name: PackageManagerFieldPackageManager,
    pub(crate) version: Version,
    integrity_check: Option<String>,
}

impl FromStr for PackageManagerField {
    type Err = PackageManagerFieldError;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val.split_once('@') {
            Some((name, remaining)) => {
                let name = PackageManagerFieldPackageManager::from_str(name).map_err(
                    |package_manager| PackageManagerFieldError::InvalidPackageManager {
                        package_manager,
                        field_value: val.to_owned(),
                    },
                )?;

                if let Some((version, integrity_check)) = remaining.split_once('+') {
                    let version = Version::from_str(version).map_err(|e| {
                        PackageManagerFieldError::Version {
                            field_value: val.to_owned(),
                            source: e,
                        }
                    })?;
                    Ok(Self {
                        name,
                        version,
                        integrity_check: Some(integrity_check.to_owned()),
                    })
                } else {
                    let version = Version::from_str(remaining).map_err(|e| {
                        PackageManagerFieldError::Version {
                            field_value: val.to_owned(),
                            source: e,
                        }
                    })?;
                    Ok(Self {
                        name,
                        version,
                        integrity_check: None,
                    })
                }
            }
            None => Err(PackageManagerFieldError::InvalidField {
                field_value: val.to_owned(),
            }),
        }
    }
}

impl Display for PackageManagerField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(integrity_check) = self.integrity_check.as_ref() {
            write!(f, "{}@{}+{integrity_check}", self.name, self.version)
        } else {
            write!(f, "{}@{}", self.name, self.version)
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum PackageManagerFieldError {
    InvalidField {
        field_value: String,
    },
    InvalidPackageManager {
        package_manager: String,
        field_value: String,
    },
    Version {
        field_value: String,
        source: VersionError,
    },
}

#[derive(Debug, PartialEq)]
pub(crate) enum PackageManagerFieldPackageManager {
    Npm,
    Pnpm,
    Yarn,
}

impl FromStr for PackageManagerFieldPackageManager {
    type Err = String;

    fn from_str(val: &str) -> Result<Self, Self::Err> {
        match val {
            "npm" => Ok(PackageManagerFieldPackageManager::Npm),
            "pnpm" => Ok(PackageManagerFieldPackageManager::Pnpm),
            "yarn" => Ok(PackageManagerFieldPackageManager::Yarn),
            _ => Err(val.to_owned()),
        }
    }
}

impl Display for PackageManagerFieldPackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PackageManagerFieldPackageManager::Npm => "npm",
                PackageManagerFieldPackageManager::Pnpm => "pnpm",
                PackageManagerFieldPackageManager::Yarn => "yarn",
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::{assert_error_snapshot, create_json_error};
    use serde_json::json;

    #[test]
    fn test_parse_package_manager_field_with_name_and_version() {
        assert_eq!(
            PackageManagerField::from_str("npm@1.2.3"),
            Ok(PackageManagerField {
                name: "npm".parse().unwrap(),
                version: Version::parse("1.2.3").unwrap(),
                integrity_check: None
            })
        );
        assert_eq!(
            PackageManagerField::from_str("pnpm@4.5.6"),
            Ok(PackageManagerField {
                name: "pnpm".parse().unwrap(),
                version: Version::parse("4.5.6").unwrap(),
                integrity_check: None
            })
        );
        assert_eq!(
            PackageManagerField::from_str("yarn@7.8.9"),
            Ok(PackageManagerField {
                name: "yarn".parse().unwrap(),
                version: Version::parse("7.8.9").unwrap(),
                integrity_check: None
            })
        );
    }

    #[test]
    fn test_parse_package_manager_field() {
        assert_eq!(
            PackageManagerField::from_str("pnpm@9.11.0+sha512.0a203ffaed5a3f63242cd064c8fb5892366c103e328079318f78062f24ea8c9d50bc6a47aa3567cabefd824d170e78fa2745ed1f16b132e16436146b7688f19b"),
            Ok(PackageManagerField {
                name: "pnpm".parse().unwrap(),
                version: Version::parse("9.11.0").unwrap(),
                integrity_check: Some("sha512.0a203ffaed5a3f63242cd064c8fb5892366c103e328079318f78062f24ea8c9d50bc6a47aa3567cabefd824d170e78fa2745ed1f16b132e16436146b7688f19b".to_string())
            })
        );
    }

    #[test]
    fn read_valid_package_with_node_engine() {
        let package_json = PackageJson(json!({
            "engines": {
                "node": "16.0.0"
            }
        }));
        assert_eq!(
            package_json.node_engine().unwrap().unwrap().to_string(),
            Requirement::parse("16.0.0").unwrap().to_string()
        );
    }

    #[test]
    fn read_error_message() {
        assert_error_snapshot(&package_json_read_error_message(
            Path::new("./package.json"),
            &std::io::Error::other("test I/O error blah"),
        ));
    }

    #[test]
    fn parse_error_message() {
        assert_error_snapshot(&package_json_parse_error_message(
            Path::new("./package.json"),
            &create_json_error(),
        ));
    }
}
