use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use thiserror::Error;

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct PackageJson {
    pub(crate) scripts: Option<Scripts>,
}

#[derive(Deserialize, Debug, Default, Clone)]
pub(crate) struct Scripts {
    pub(crate) start: Option<String>,
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
    /// Determines if a given `PackageJson` has a start script defined
    pub(crate) fn has_start_script(&self) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|scripts| scripts.start.is_some())
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
}
