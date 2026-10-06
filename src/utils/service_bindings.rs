use crate::BuildpackError;
use crate::utils::error_handling::error_message;
use indoc::formatdoc;
use libcnb::Env;
use std::collections::BTreeMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const SERVICE_BINDING_ROOT_ENV: &str = "SERVICE_BINDING_ROOT";
const HOME_ENV: &str = "HOME";
const DEFAULT_BINDINGS_DIR: &str = "/platform/bindings";
const NPMRC_RELATIVE_TO_HOME: &str = ".npmrc";
const YARNRC_RELATIVE_TO_HOME: &str = ".yarnrc";
const PNPM_AUTH_INI_RELATIVE_TO_HOME: &str = ".config/pnpm/auth.ini";

#[derive(Debug, Clone)]
pub(crate) struct Bindings {
    pub(crate) bindings: Vec<Binding>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct Binding {
    pub(crate) name: String,
    pub(crate) provider: Option<String>,
    pub(crate) binding_type: Option<String>,
    pub(crate) path: PathBuf,
    pub(crate) secrets: BTreeMap<String, String>,
}
impl Binding {
    fn resolve(path: PathBuf) -> Result<Self, BuildpackError> {
        let binding_name = path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| create_binding_name_error(&path))?
            .to_string();

        let mut binding_type = None;
        let mut provider = None;
        let mut secrets = BTreeMap::new();

        let entries = fs::read_dir(&path).map_err(|e| create_read_binding_dir_error(&path, e))?;
        for entry in entries {
            let binding_entry = entry.map_err(|e| create_read_binding_dir_error(&path, e))?;
            let binding_path = binding_entry.path();

            if binding_path.is_dir() {
                continue;
            }

            let Some(file_name) = binding_path.file_name().and_then(OsStr::to_str) else {
                tracing::warn!(
                    "Skipping service binding file with a non-UTF-8 name at {}",
                    binding_path.display()
                );
                continue;
            };

            match file_name {
                "type" => {
                    binding_type = Some(read_binding_value(&binding_path)?.trim().to_string());
                }
                "provider" => {
                    provider = Some(read_binding_value(&binding_path)?.trim().to_string());
                }
                _ => {
                    secrets.insert(file_name.to_string(), read_binding_value(&binding_path)?);
                }
            }
        }

        Ok(Self {
            name: binding_name,
            provider,
            binding_type,
            path,
            secrets,
        })
    }
}

impl Bindings {
    fn load() -> Result<Self, BuildpackError> {
        let bindings_dir = get_bindings_dir();

        if !bindings_dir.exists() {
            tracing::debug!(
                "No bindings directory found at {} (SERVICE_BINDING_ROOT={:?})",
                bindings_dir.display(),
                env::var(SERVICE_BINDING_ROOT_ENV).ok()
            );
            return Ok(Self {
                bindings: Vec::new(),
            });
        }

        if !bindings_dir.is_dir() {
            tracing::warn!(
                "Bindings directory path {} is not a directory",
                bindings_dir.display()
            );
            return Ok(Self {
                bindings: Vec::new(),
            });
        }

        let bindings = fs::read_dir(&bindings_dir)
            .map_err(|e| create_read_binding_dir_error(&bindings_dir, e))?
            .filter_map(|entry| match entry {
                Ok(entry) => {
                    let binding_path = entry.path();
                    if binding_path.is_dir() {
                        Some(Binding::resolve(binding_path))
                    } else {
                        None
                    }
                }
                Err(error) => {
                    tracing::warn!(
                        "Failed to read service binding entry in {} with error: {}",
                        bindings_dir.display(),
                        error
                    );
                    None
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { bindings })
    }

    pub(crate) fn find_by_type(&self, binding_type: &str) -> Vec<&Binding> {
        self.bindings
            .iter()
            .filter(|binding| binding.binding_type.as_deref() == Some(binding_type))
            .collect()
    }

    pub(crate) fn find_by_name(&self, binding_name: &str) -> Vec<&Binding> {
        self.bindings
            .iter()
            .filter(|binding| binding.name == binding_name)
            .collect()
    }
}

fn get_bindings_dir() -> PathBuf {
    env::var(SERVICE_BINDING_ROOT_ENV)
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_BINDINGS_DIR))
}

/// Applies npm service bindings by making package-manager configuration files available during the build.
pub(crate) fn apply_package_manager_bindings(env: &Env) -> Result<(), BuildpackError> {
    let bindings = Bindings::load()?;
    let home_dir = env
        .get(HOME_ENV)
        .map(PathBuf::from)
        .unwrap();

    let npm_binding = bindings
        .find_by_type("npm")
        .into_iter()
        .next()
        .or_else(|| bindings.find_by_name("npm").into_iter().next());
    if let Some(npm) = npm_binding {
        let npmrc_dest = home_dir.join(NPMRC_RELATIVE_TO_HOME);
        write_binding_secret(npm, "npm", ".npmrc", &npmrc_dest)?;
    }

    let yarn_binding = bindings
        .find_by_type("yarn")
        .into_iter()
        .next()
        .or_else(|| bindings.find_by_name("yarn").into_iter().next());
    if let Some(yarn) = yarn_binding {
        let yarnrc_dest = home_dir.join(YARNRC_RELATIVE_TO_HOME);
        write_binding_secret(yarn, "yarn", ".yarnrc", &yarnrc_dest)?;
    }

    let pnpm_binding = bindings
        .find_by_type("pnpm")
        .into_iter()
        .next()
        .or_else(|| bindings.find_by_name("pnpm").into_iter().next());
    if let Some(pnpm) = pnpm_binding {
        let pnpm_auth_ini_dest = home_dir.join(PNPM_AUTH_INI_RELATIVE_TO_HOME);
        write_binding_secret(pnpm, "pnpm", "auth.ini", &pnpm_auth_ini_dest)?;
    }

    Ok(())
}

fn write_binding_secret(
    binding: &Binding,
    binding_name: &str,
    secret_name: &str,
    path: &Path,
) -> Result<(), BuildpackError> {
    let Some(secret) = binding.secrets.get(secret_name) else {
        tracing::warn!(
            "Skipping {} binding {} because it does not define {}",
            binding_name,
            binding.name,
            secret_name
        );
        return Ok(());
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| create_write_binding_error(binding_name, secret_name, path, e))?;
    }

    fs::write(path, secret)
        .map_err(|e| create_write_binding_error(binding_name, secret_name, path, e))?;

    Ok(())
}

fn read_binding_value(path: &Path) -> Result<String, BuildpackError> {
    fs::read_to_string(path).map_err(|e| create_read_binding_file_error(path, e))
}

fn create_binding_name_error(path: &Path) -> BuildpackError {
    error_message()
        .id("service_binding/invalid_binding_name")
        .error_type(crate::utils::error_handling::ErrorType::Internal)
        .header("Failed to resolve service binding name")
        .body(formatdoc! {
            "
            Could not determine the service binding name from the binding directory path.

            Path: {}

            Suggestions:
            - Ensure the binding directory has a valid name
            - Check the SERVICE_BINDING_ROOT path
            ",
            path.display()
        })
        .create()
        .into()
}

fn create_read_binding_file_error(path: &Path, error: std::io::Error) -> BuildpackError {
    error_message()
        .id("service_binding/read_file")
        .error_type(crate::utils::error_handling::ErrorType::Internal)
        .header("Failed to read service binding file")
        .body(formatdoc! {
            "
            Could not read a service binding file while loading package-manager configuration.

            Path: {}

            Suggestions:
            - Ensure the binding files are readable
            - Check the service binding mount permissions
            ",
            path.display()
        })
        .debug_info(error.to_string())
        .create()
        .into()
}

fn create_read_binding_dir_error(path: &Path, error: std::io::Error) -> BuildpackError {
    error_message()
        .id("service_binding/read_dir")
        .error_type(crate::utils::error_handling::ErrorType::Internal)
        .header("Failed to read service binding directory")
        .body(formatdoc! {
            "
            Could not read the service binding directory.

            Path: {}

            Suggestions:
            - Ensure the binding directory exists and is accessible
            - Check the SERVICE_BINDING_ROOT value
            ",
            path.display()
        })
        .debug_info(error.to_string())
        .create()
        .into()
}

fn create_write_binding_error(
    binding: &str,
    secret_name: &str,
    path: &Path,
    error: std::io::Error,
) -> BuildpackError {
    let path_display = path.display();
    error_message()
        .id(format!("{}_binding/write_{}", binding, secret_name))
        .error_type(crate::utils::error_handling::ErrorType::Internal)
        .header(format!("Failed to write {} configuration", binding))
        .body(formatdoc! {
            "
            Could not write the {} configuration file from the service binding.

            Path: {}

            Suggestions:
            - Ensure the home directory is writable
            - Check file system permissions
            ",
            binding,
            path_display
        })
        .debug_info(error.to_string())
        .create()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn binding_resolve_reads_type_and_provider() {
        let temp_dir = TempDir::new().unwrap();
        let binding_dir = temp_dir.path().join("npm");
        std::fs::create_dir(&binding_dir).unwrap();
        std::fs::write(binding_dir.join("type"), "npm\n").unwrap();
        std::fs::write(binding_dir.join("provider"), "service\n").unwrap();
        std::fs::write(binding_dir.join(".npmrc"), "//registry.example.com\n").unwrap();

        let binding = Binding::resolve(binding_dir.clone()).unwrap();

        assert_eq!(binding.binding_type.as_deref(), Some("npm"));
        assert_eq!(binding.provider.as_deref(), Some("service"));
        assert_eq!(binding.path, binding_dir);
        assert_eq!(
            binding.secrets.get(".npmrc").map(String::as_str),
            Some("//registry.example.com\n")
        );
    }

    #[test]
    fn binding_lookup_prefers_type_before_name_fallback() {
        let typed_binding = Binding {
            name: "typed-npm".into(),
            provider: None,
            binding_type: Some("npm".into()),
            path: PathBuf::from("/tmp/typed-npm"),
            secrets: BTreeMap::new(),
        };
        let named_binding = Binding {
            name: "npm".into(),
            provider: None,
            binding_type: None,
            path: PathBuf::from("/tmp/npm"),
            secrets: BTreeMap::new(),
        };
        let bindings = Bindings {
            bindings: vec![typed_binding, named_binding],
        };

        let types = bindings.find_by_type("npm");
        let names = bindings.find_by_name("npm");

        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "typed-npm");
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].name, "npm");
    }

    #[test]
    fn write_binding_secret_creates_parent_directories_and_writes_file() {
        let home_dir = TempDir::new().unwrap();
        let binding = Binding {
            name: "npm".into(),
            provider: Some("service".into()),
            binding_type: Some("npm".into()),
            path: home_dir.path().join("npm"),
            secrets: BTreeMap::from([(".npmrc".into(), "//registry.example.com\n".into())]),
        };
        let target_path = home_dir.path().join(".npmrc");

        write_binding_secret(&binding, "npm", ".npmrc", &target_path).unwrap();

        assert_eq!(
            std::fs::read_to_string(&target_path).unwrap(),
            "//registry.example.com\n"
        );
    }
}
