use crate::versions::Ver;
use serde::{Deserialize};
use std::io::BufReader;
use std::fs::File;
use std::path::Path;

fn read<P: AsRef<Path>>(path: P) -> Result<Package, PackageError> {
    let file = match File::open(path) {
        Err(e) => return Err(PackageError(format!("could not open package.json: {}", e))),
        Ok(f) => f
    };
    let rdr = BufReader::new(file);

    return match serde_json::from_reader(rdr) {
        Ok(p) => Ok(p),
        Err(e) => Err(PackageError(format!("could not parse package.json: {}", e))),
    }
}

#[derive(Deserialize,Debug)]
struct Package {
    name: String,
    version: Ver,
    engines: Option<Engines>,
}

#[derive(Deserialize,Debug)]
struct Engines {
    node: Option<Ver>,
    yarn: Option<Ver>,
    npm: Option<Ver>,
}

#[derive(Deserialize,Debug,PartialEq)]
struct PackageError(String);

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::Builder;
    use std::io::{Write};

    #[test]
    fn read_valid_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{}", "{\"name\": \"foo\",\"version\": \"0.0.0\"}").unwrap();
        let pkg = read(f.path()).unwrap();
        assert_eq!(pkg.name, "foo");
        assert_eq!(pkg.version.to_string(), "0.0.0");
    }

    #[test]
    fn read_valid_package_with_node_engine() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{}", "{
            \"name\": \"foo\",
            \"version\": \"0.0.0\",
            \"engines\": {
                \"node\": \"16.0.0\"
            }
        }").unwrap();
        let pkg = read(f.path()).unwrap();
        assert_eq!(&pkg.engines.unwrap().node.unwrap().to_string(), "16.0.0")
    }

    #[test]
    fn read_missing_package() {
        let res = read(Path::new("/over/there/package.json"));
        assert!(res.unwrap_err().0.contains("could not open package.json"));
    }

    #[test]
    fn read_invalid_package() {
        let mut f = Builder::new().tempfile().unwrap();
        write!(f, "{}", "{").unwrap();
        let res = read(f.path());
        assert!(res.unwrap_err().0.contains("could not parse package.json"));
    }
}
