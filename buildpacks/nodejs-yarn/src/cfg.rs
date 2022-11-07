use heroku_nodejs_utils::{package_json::PackageJson, vrs::Requirement};
use libherokubuildpack::log::log_info;
use std::path::Path;

// Reads parsed `engines.yarn` requirement from a `PackageJson`.
pub(crate) fn requested_yarn_range(pkg_json: &PackageJson) -> Requirement {
    pkg_json
        .engines
        .as_ref()
        .and_then(|e| {
            e.yarn.as_ref().map(|v| {
                log_info(format!(
                    "Detected yarn version range {} from package.json",
                    v
                ));
                v.clone()
            })
        })
        .unwrap_or_else(|| {
            log_info("Detected no yarn version range requirement");
            Requirement::any()
        })
}

// A yarn cache is populated if it exists, and has non-hidden files.
pub(crate) fn cache_populated(cache_path: &Path) -> bool {
    cache_path
        .read_dir()
        .map(|mut contents| {
            contents.any(|entry| {
                entry
                    .map(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

// Fetches the build scripts from a `PackageJson` and returns them in priority order
pub(crate) fn get_build_scripts(pkg_json: &PackageJson) -> Option<Vec<String>> {
    let mut scripts = vec![];
    if let Some(s) = &pkg_json.scripts {
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
    scripts.is_empty().then_some(scripts)
}
