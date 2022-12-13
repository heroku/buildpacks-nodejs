use heroku_nodejs_utils::package_json::PackageJson;

/// Return the package manager name from package.json if it's present and
/// supported.
pub(crate) fn get_supported_package_manager(pkg_json: &PackageJson) -> Option<String> {
    let pkg_mgr_name = pkg_json.package_manager.clone()?.name;
    match pkg_mgr_name.as_str() {
        "yarn" => Some(pkg_mgr_name),
        _ => None,
    }
}
