use heroku_nodejs_utils::package_json::{PackageJson, PackageJsonError};
use heroku_nodejs_utils::vrs::{Requirement, Version};
use libcnb::{read_toml_file, Env};
use libherokubuildpack::toml::toml_select_value;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

pub(crate) fn is_function<P>(d: P) -> bool
where
    P: Into<PathBuf>,
{
    let dir = d.into();
    dir.join("function.toml").exists() || {
        read_toml_file(dir.join("project.toml"))
            .ok()
            .and_then(|toml: toml::Value| {
                toml_select_value(vec!["com", "salesforce", "type"], &toml)
                    .and_then(toml::Value::as_str)
                    .map(|value| value == "function")
            })
            .unwrap_or(false)
    }
}

pub(crate) fn get_main<P>(d: P) -> Result<PathBuf, MainError>
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

pub(crate) fn get_declared_runtime_package_version<P>(
    app_dir: P,
    package_name: &String,
) -> Result<Option<String>, ExplicitRuntimeDependencyError>
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
        .and_then(|dependencies| dependencies.get(package_name).cloned()))
}

pub(crate) fn check_minumum_node_version<P>(app_dir: P) -> Result<(), MinimumNodeVersionError>
where
    P: Into<PathBuf>,
{
    let minimum_version = Requirement::parse(">=16").expect("The minimum version should be valid.");

    let version = Command::new("node")
        .arg("--version")
        .envs(&Env::from_current())
        .current_dir(app_dir.into())
        .output()
        .map_err(MinimumNodeVersionError::Command)
        .and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            stdout
                .parse::<Version>()
                .map_err(|e| MinimumNodeVersionError::ParseVersion(stdout, e))
        })?;

    if minimum_version.satisfies(&version) {
        Ok(())
    } else {
        Err(MinimumNodeVersionError::DoesNotMeetMinimumRequirement(
            minimum_version,
            version,
        ))
    }
}

#[derive(Error, Debug)]
pub(crate) enum MainError {
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
pub(crate) enum ExplicitRuntimeDependencyError {
    #[error("Failure while attempting to read from package.json. {0}")]
    PackageJson(#[from] PackageJsonError),
    #[error("The '{package_name}' package must be declared in the 'dependencies' field of your package.json but was found in 'devDependencies'.")]
    DeclaredAsDevDependency { package_name: String },
}

#[derive(Error, Debug)]
pub(crate) enum MinimumNodeVersionError {
    #[error("Failure while attempting to parse `node --version` output\nOutput: {0}\nError: {1}")]
    ParseVersion(String, heroku_nodejs_utils::vrs::VersionError),
    #[error("Failure while attempting to execute `node --version`\nError: {0}")]
    Command(std::io::Error),
    #[error("The minimum required version of Node.js is {0} but version {1} is installed. Please update the `engines.node` field in your package.json to a newer version.")]
    DoesNotMeetMinimumRequirement(Requirement, Version),
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
        let mut file = File::create(file_path).expect("Couldn't create temp project descriptor");
        write!(file, "{file_contents}").expect("Couldn't write to temp project descriptor");
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
        let err = get_declared_runtime_package_version(
            dir.path(),
            &String::from("@heroku/sf-fx-runtime-nodejs"),
        )
        .expect_err("this should have throw an error");
        assert!(err
            .to_string()
            .contains("The '@heroku/sf-fx-runtime-nodejs' package must be declared in the 'dependencies' field of your package.json but was found in 'devDependencies'."));
    }

    #[test]
    fn get_explicit_dependency_when_package_json_in_invalid() {
        let package_name = String::from("@heroku/sf-fx-runtime-nodejs");
        let dir = create_dir_with_file("package.json", "{\"name\": \"test....}");
        let err = get_declared_runtime_package_version(dir.path(), &package_name)
            .expect_err("this should have thrown an error");
        assert!(err
            .to_string()
            .contains("Failure while attempting to read from package.json."));
    }
}
