use crate::vrs::{Requirement, Version};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<Version>,
    pub engines: Option<Engines>,
    pub scripts: Option<Scripts>,
    pub main: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(default, rename = "packageManager")]
    pub package_manager: Option<PackageManager>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Engines {
    pub node: Option<Requirement>,
    pub yarn: Option<Requirement>,
    pub npm: Option<Requirement>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Scripts {
    pub start: Option<String>,
    pub build: Option<String>,
    #[serde(rename = "heroku-prebuild")]
    pub heroku_prebuild: Option<String>,
    #[serde(rename = "heroku-build")]
    pub heroku_build: Option<String>,
    #[serde(rename = "heroku-postbuild")]
    pub heroku_postbuild: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "String")]
pub struct PackageManager {
    pub name: String,
    pub version: Version,
}

impl TryFrom<String> for PackageManager {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let err_msg = format!("Couldn't parse `packageManager` value: \"{value}\".");

        value
            .split_once('@')
            .ok_or(err_msg.clone())
            .and_then(|(name_str, version_str)| {
                if name_str.is_empty() {
                    Err(format!(
                    "{err_msg} Hint: Does the value have a package manager name before the \"@\"?"
                ))
                } else {
                    Version::from_str(version_str)
                        .map_err(|error| error.to_string())
                        .map(|version| PackageManager {
                            name: name_str.to_string(),
                            version,
                        })
                }
            })
    }
}

#[derive(Error, Debug)]
pub enum PackageJsonError {
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
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, PackageJsonError> {
        let file = File::open(path).map_err(PackageJsonError::AccessError)?;
        let rdr = BufReader::new(file);
        serde_json::from_reader(rdr).map_err(PackageJsonError::ParseError)
    }

    #[must_use]
    /// Fetches the build scripts from a `PackageJson` and returns them in priority order
    pub fn build_scripts(&self) -> Vec<String> {
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
    pub fn has_start_script(&self) -> bool {
        self.scripts
            .as_ref()
            .map_or(false, |scripts| scripts.start.is_some())
    }
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
        let pkg = PackageJson::read(f.path()).unwrap();
        assert_eq!(pkg.name, None);
        assert_eq!(pkg.version, None);
    }

    #[test]
    fn read_valid_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{{\"name\": \"foo\",\"version\": \"0.0.0\"}}").unwrap();
        let pkg = PackageJson::read(f.path()).unwrap();
        assert_eq!(pkg.name.unwrap(), "foo");
        assert_eq!(pkg.version.unwrap().to_string(), "0.0.0");
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
