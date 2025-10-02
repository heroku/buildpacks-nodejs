use crate::utils::package_json::PackageJson;

/// Return the package manager name from package.json if it's present and
/// supported.
pub(crate) fn get_supported_package_manager(pkg_json: &PackageJson) -> Option<String> {
    let pkg_mgr_name = pkg_json.package_manager.clone()?.name;
    match pkg_mgr_name.as_str() {
        "yarn" | "pnpm" | "npm" => Some(pkg_mgr_name),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{package_json::PackageManager, vrs::Version};

    #[test]
    fn test_get_supported_package_manager_yarn() {
        let pkg_json = PackageJson {
            package_manager: Some(PackageManager {
                name: "yarn".to_owned(),
                version: Version::parse("3.1.2").unwrap(),
            }),
            ..PackageJson::default()
        };
        let pkg_mgr =
            get_supported_package_manager(&pkg_json).expect("Expected to get a package manager");
        assert_eq!("yarn", pkg_mgr);
    }

    #[test]
    fn test_get_supported_package_manager_pnpm() {
        let pkg_json = PackageJson {
            package_manager: Some(PackageManager {
                name: "pnpm".to_owned(),
                version: Version::parse("8.1.0").unwrap(),
            }),
            ..PackageJson::default()
        };
        let pkg_mgr =
            get_supported_package_manager(&pkg_json).expect("Expected to get a package manager");
        assert_eq!("pnpm", pkg_mgr);
    }

    #[test]
    fn test_get_supported_package_manager_other() {
        let pkg_json = PackageJson {
            package_manager: Some(PackageManager {
                name: "other-package-manager".to_owned(),
                version: Version::parse("1.0.0").unwrap(),
            }),
            ..PackageJson::default()
        };
        assert!(get_supported_package_manager(&pkg_json).is_none());
    }
}
