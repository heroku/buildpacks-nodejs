use crate::versions::{Requirement, Version};
use serde::Deserialize;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct PackageJson {
    pub name: String,
    pub version: Version,
    pub engines: Option<Engines>,
}

#[derive(Deserialize, Debug)]
pub struct Engines {
    pub node: Option<Requirement>,
    pub yarn: Option<Requirement>,
    pub npm: Option<Requirement>,
}

#[derive(Debug)]
pub enum PackageJsonError {
    AccessError(std::io::Error),
    ParseError(serde_json::Error),
}

impl PackageJson {
    /// Reads a package.json from a path into a `PackageJson` struct.
    ///
    /// # Errors
    ///
    /// * Invalid/malformed JSON
    /// * Path does not exist or is unreadable
    /// * Versionsion strings are invalid/malformed
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, PackageJsonError> {
        let file = File::open(path).map_err(PackageJsonError::AccessError)?;
        let rdr = BufReader::new(file);
        serde_json::from_reader(rdr).map_err(PackageJsonError::ParseError)
    }
}

impl fmt::Display for PackageJsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AccessError(e) => write!(f, "Could not read package.json: {}", e),
            Self::ParseError(e) => write!(f, "Could not parse package.json: {}", e),
        }
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
        assert_eq!(pkg.version.to_string(), "0.0.0");
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
        assert_eq!(&pkg.engines.unwrap().node.unwrap().to_string(), "16.0.0")
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
