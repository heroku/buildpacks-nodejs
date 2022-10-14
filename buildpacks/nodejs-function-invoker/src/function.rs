use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use libcnb::read_toml_file;
use libherokubuildpack::toml::toml_select_value;
use std::path::PathBuf;
use thiserror::Error;
use toml::Value;

pub fn is_function<P>(d: P) -> bool
where
    P: Into<PathBuf>,
{
    let dir = d.into();
    dir.join("function.toml").exists() || {
        read_toml_file(dir.join("project.toml"))
            .ok()
            .and_then(|toml: Value| {
                toml_select_value(vec!["com", "salesforce", "type"], &toml)
                    .and_then(toml::Value::as_str)
                    .map(|value| value == "function")
            })
            .unwrap_or(false)
    }
}

pub fn get_main<P>(d: P) -> Result<PathBuf, MainError>
where
    P: Into<PathBuf>,
{
    let dir: PathBuf = d.into();
    PackageJson::read(dir.join("package.json"))
        .map_err(MainError::PackageJson)
        .and_then(|pjson| pjson.main.ok_or(MainError::MissingKey))
        .map(|main| dir.join(main))
        .and_then(|path| path.exists().then_some(path).ok_or(MainError::MissingFile))
}

pub fn get_declared_runtime_package<P>(
    app_dir: P,
    package_name: &String,
) -> Result<Option<(&String, String)>, ExplicitRuntimeDependencyError>
where
    P: Into<PathBuf>,
{
    let package_json = PackageJson::read(app_dir.into().join("package.json"))
        .map_err(ExplicitRuntimeDependencyError::PackageJson)?;

    if let Some(dev_dependencies) = package_json.dev_dependencies {
        if dev_dependencies.contains_key(package_name) {
            return Err(ExplicitRuntimeDependencyError::DeclaredAsDevDependency {
                package_name: package_name.clone(),
            });
        }
    }

    Ok(package_json
        .dependencies
        .and_then(|dependencies| dependencies.get(package_name).cloned())
        .map(|version| (package_name, version)))
}

#[derive(Error, Debug)]
pub enum MainError {
    #[error("Could not determine function file location from package.json. {0}")]
    PackageJson(#[from] PackageJsonError),
    #[error(
        "Key `main` missing from package.json. Ensure `main` references function file location."
    )]
    MissingKey,
    #[error("File referenced by `main` in package.json could not be found. Ensure `main` references function file location.")]
    MissingFile,
}

#[derive(Error, Debug)]
pub enum ExplicitRuntimeDependencyError {
    #[error("Failure while attempting to read from package.json. {0}")]
    PackageJson(#[from] PackageJsonError),
    #[error("The '{package_name}' package must be declared in the 'dependencies' field of your package.json but was found in 'devDependencies'.")]
    DeclaredAsDevDependency { package_name: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use tempfile::{tempdir, TempDir};

    fn create_dir_with_file(file_name: &str, file_contents: &str) -> TempDir {
        let dir = tempdir().expect("Couldn't create temp dir");
        let path: PathBuf = dir.path().into();
        let file_path = path.join(file_name);
        let mut file = File::create(&file_path).expect("Couldn't create temp project descriptor");
        write!(file, "{}", file_contents).expect("Couldn't write to temp project descriptor");
        dir
    }

    #[test]
    fn is_function_with_function_toml() {
        let dir = create_dir_with_file("function.toml", "");
        assert!(is_function(dir.path()));
    }

    #[test]
    fn is_function_with_project_toml() {
        let dir = create_dir_with_file("project.toml", "[com.salesforce]\n type = \"function\"");
        assert!(is_function(dir.path()));
    }

    #[test]
    fn is_function_with_different_type() {
        let dir = create_dir_with_file("project.toml", "[com.salesforce]\n type = \"wubalubdub\"");
        assert!(!is_function(dir.path()));
    }

    #[test]
    fn is_function_missing_descriptor() {
        let dir = create_dir_with_file("package.json", "{}");
        assert!(!is_function(dir.path()));
    }

    #[test]
    fn get_main_exists() {
        let package_json = json!({
            "name": "test-main-function",
            "main": "index.js"
        });
        let dir = create_dir_with_file("package.json", package_json.to_string().as_str());
        let index_path = dir.path().join("index.js");
        File::create(&index_path).expect("Could not create temp index.js");
        let main_path = get_main(dir.path()).unwrap();
        assert_eq!(main_path, index_path);
    }

    #[test]
    fn get_main_no_file() {
        let package_json = json!({
            "name": "test-main-function",
            "main": "index.js"
        });
        let dir = create_dir_with_file("package.json", package_json.to_string().as_str());
        let err = get_main(dir.path()).expect_err("found main function when there wasn't a file");
        assert!(err
            .to_string()
            .contains("File referenced by `main` in package.json could not be found."));
    }

    #[test]
    fn get_main_no_key() {
        let package_json = json!({
            "name": "test-main-function"
        });
        let dir = create_dir_with_file("package.json", package_json.to_string().as_str());
        let err =
            get_main(dir.path()).expect_err("found main function when there wasn't a main key");
        assert!(err
            .to_string()
            .contains("Key `main` missing from package.json"));
    }

    #[test]
    fn get_main_bad_json() {
        let dir = create_dir_with_file("package.json", "{\"name\": \"test....}");
        let err = get_main(dir.path())
            .expect_err("found main function when the package.json was malformed");
        assert!(err
            .to_string()
            .contains("Could not determine function file location from package.json. Could not parse package.json"));
    }

    #[test]
    fn get_explicit_dependency_when_declared_as_dev_dependency() {
        let package_json = json!({
            "name": "test",
            "devDependencies": {
                "@heroku/sf-fx-runtime-nodejs": "0.0.0",
            }
        });
        let dir = create_dir_with_file("package.json", package_json.to_string().as_str());
        let err =
            get_declared_runtime_package(dir.path(), &String::from("@heroku/sf-fx-runtime-nodejs"))
                .expect_err("this should have throw an error");
        assert!(err
            .to_string()
            .contains("The '@heroku/sf-fx-runtime-nodejs' package must be declared in the 'dependencies' field of your package.json but was found in 'devDependencies'."));
    }

    #[test]
    fn get_explicit_dependency_when_package_json_in_invalid() {
        let package_name = String::from("@heroku/sf-fx-runtime-nodejs");
        let dir = create_dir_with_file("package.json", "{\"name\": \"test....}");
        let err = get_declared_runtime_package(dir.path(), &package_name)
            .expect_err("this should have thrown an error");
        assert!(err
            .to_string()
            .contains("Failure while attempting to read from package.json."));
    }
}
