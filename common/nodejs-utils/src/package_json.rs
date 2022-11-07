use crate::vrs::{Requirement, Version};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

#[derive(Deserialize, Debug, Clone)]
pub struct PackageJson {
    pub name: String,
    pub version: Option<Version>,
    pub engines: Option<Engines>,
    pub scripts: Option<Scripts>,
    pub main: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Engines {
    pub node: Option<Requirement>,
    pub yarn: Option<Requirement>,
    pub npm: Option<Requirement>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Scripts {
    pub start: Option<String>,
    pub build: Option<String>,
    pub heroku_prebuild: Option<String>,
    pub heroku_build: Option<String>,
    pub heroku_postbuild: Option<String>,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::Builder;

    #[test]
    fn read_valid_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{{\"name\": \"foo\",\"version\": \"0.0.0\"}}").unwrap();
        let pkg = PackageJson::read(f.path()).unwrap();
        assert_eq!(pkg.name, "foo");
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
    fn read_missing_package() {
        let res = PackageJson::read(Path::new("/over/there/package.json"));
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        println!("{}", err);
        assert!(err.contains("Could not read package.json"));
    }

    #[test]
    fn read_invalid_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{{").unwrap();
        let res = PackageJson::read(f.path());
        assert!(res.is_err());
        let err = res.unwrap_err().to_string();
        println!("{}", err);
        assert!(err.contains("Could not parse package.json"));
    }
}
