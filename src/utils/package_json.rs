use crate::utils::vrs::{Requirement, Version};
use serde::{Deserialize, Deserializer, de};
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PackageJson {
    pub(crate) engines: Option<Engines>,
    pub(crate) scripts: Option<Scripts>,
    #[serde(
        default,
        deserialize_with = "deserialize_package_manager",
        rename = "packageManager"
    )]
    pub(crate) package_manager: Option<PackageManager>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct Engines {
    pub(crate) node: Option<Requirement>,
    pub(crate) npm: Option<Requirement>,
    pub(crate) yarn: Option<Requirement>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct Scripts {
    pub(crate) start: Option<String>,
    pub(crate) build: Option<String>,
    #[serde(rename = "heroku-prebuild")]
    pub(crate) heroku_prebuild: Option<String>,
    #[serde(rename = "heroku-build")]
    pub(crate) heroku_build: Option<String>,
    #[serde(rename = "heroku-postbuild")]
    pub(crate) heroku_postbuild: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct PackageManager {
    pub(crate) name: String,
    pub(crate) version: Version,
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}@{}", self.name, self.version)
    }
}

#[derive(Error, Debug)]
pub(crate) enum PackageJsonError {
    #[error("Could not read package.json. {0}")]
    AccessError(std::io::Error),
    #[error("Could not parse package.json. {0}")]
    ParseError(serde_json::Error),
}

impl PackageJson {
    /// Reads a package.json from a path into a `PackageJson` struct.
    ///
    /// # Errors
    ///
    /// * Invalid/malformed JSON
    /// * Path does not exist or is unreadable
    /// * Version strings are invalid/malformed
    pub(crate) fn read<P: AsRef<Path>>(path: P) -> Result<Self, PackageJsonError> {
        let file = File::open(path).map_err(PackageJsonError::AccessError)?;
        let rdr = BufReader::new(file);
        serde_json::from_reader(rdr).map_err(PackageJsonError::ParseError)
    }

    #[must_use]
    /// Fetches the build scripts from a `PackageJson` and returns them in priority order
    pub(crate) fn build_scripts(&self) -> Vec<String> {
        let mut scripts = vec![];
        if let Some(s) = &self.scripts {
            if s.heroku_prebuild.is_some() {
                scripts.push("heroku-prebuild".to_owned());
            }
            if s.heroku_build.is_some() {
                scripts.push("heroku-build".to_owned());
            } else if s.build.is_some() {
                scripts.push("build".to_owned());
            }
            if s.heroku_postbuild.is_some() {
                scripts.push("heroku-postbuild".to_owned());
            }
        }
        scripts
    }

    #[must_use]
    /// Determines if a given `PackageJson` has a start script defined
    pub(crate) fn has_start_script(&self) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|scripts| scripts.start.is_some())
    }
}

/// Deserializes a `packageManager` field value (like "yarn@1.22.19" into a `Option<PackageManager>`)
fn deserialize_package_manager<'de, D>(deserializer: D) -> Result<Option<PackageManager>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: String = Deserialize::deserialize(deserializer)?;
    if value.is_empty() {
        return Ok(None);
    }
    let mut parts = value.split('@');
    let err_msg = format!("Couldn't parse `packageManager` value: \"{value}\".");
    let name = parts
        .next()
        .ok_or_else(|| de::Error::custom(&err_msg))?
        .to_string();

    if name.is_empty() {
        return Err(de::Error::custom(format!(
            "{err_msg} Hint: Does the value have a package manager name before the \"@\"?"
        )));
    }

    let vrs_str = parts.next().ok_or_else(|| {
        de::Error::custom(format!(
            "{err_msg} Hint: Does the value contain an \"@\" followed by a semantic version number?"
        ))
    })?;

    let version = Version::from_str(vrs_str).map_err(de::Error::custom)?;
    Ok(Some(PackageManager { name, version }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder;

    #[test]
    fn read_empty_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{{ }}").unwrap();
        let pkg = PackageJson::read(f.path());
        assert!(pkg.is_ok());
    }

    #[test]
    fn read_valid_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{{\"name\": \"foo\",\"version\": \"0.0.0\"}}").unwrap();
        let pkg = PackageJson::read(f.path());
        assert!(pkg.is_ok());
    }

    #[test]
    fn read_valid_package_with_node_engine() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(
            f,
            "{{
            \"name\": \"foo\",
            \"version\": \"0.0.0\",
            \"engines\": {{
                \"node\": \"16.0.0\"
            }}
        }}"
        )
        .unwrap();
        let pkg = PackageJson::read(f.path()).unwrap();
        assert_eq!(&pkg.engines.unwrap().node.unwrap().to_string(), "16.0.0");
    }

    #[test]
    fn read_valid_package_with_package_manager() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(
            f,
            "{{
            \"name\": \"foo\",
            \"packageManager\": \"yarn@3.2.0\"
            }}"
        )
        .unwrap();
        let pkg = PackageJson::read(f.path()).unwrap();
        let pkg_mgr = &pkg
            .package_manager
            .expect("Expected packageManager to exist");
        assert_eq!(pkg_mgr.name, "yarn");
        assert_eq!(pkg_mgr.version.to_string(), "3.2.0");
    }

    #[test]
    fn read_invalid_package_package_manager_no_at_version() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(
            f,
            "{{
            \"name\": \"foo\",
            \"packageManager\": \"some-package-manager\"
            }}"
        )
        .unwrap();
        let err = PackageJson::read(f.path())
            .expect_err("expected some-package-manager to cause an error")
            .to_string();
        println!("{err}");
        assert!(err.contains("some-package-manager"));
    }

    #[test]
    fn read_missing_package() {
        let res = PackageJson::read(Path::new("/over/there/package.json"));
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        println!("{err}");
        assert!(err.contains("Could not read package.json"));
    }

    #[test]
    fn read_invalid_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{{").unwrap();
        let res = PackageJson::read(f.path());
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        println!("{err}");
        assert!(err.contains("Could not parse package.json"));
    }

    #[test]
    fn test_build_scripts_all() {
        let pkg_json = PackageJson {
            scripts: Some(Scripts {
                build: Some("echo 'build'".to_owned()),
                heroku_prebuild: Some("echo 'heroku-prebuild'".to_owned()),
                heroku_build: Some("echo 'build'".to_owned()),
                heroku_postbuild: Some("echo 'heroku-postbuild'".to_owned()),
                ..Scripts::default()
            }),
            ..PackageJson::default()
        };
        let build_scripts = pkg_json.build_scripts();

        assert_eq!("heroku-prebuild", build_scripts[0]);
        assert_eq!("heroku-build", build_scripts[1]);
        assert_eq!("heroku-postbuild", build_scripts[2]);
    }

    #[test]
    fn test_build_scripts_build_only() {
        let pkg_json = PackageJson {
            scripts: Some(Scripts {
                build: Some("echo 'build'".to_owned()),
                ..Scripts::default()
            }),
            ..PackageJson::default()
        };
        let build_scripts = pkg_json.build_scripts();

        assert_eq!("build", build_scripts[0]);
    }
}
