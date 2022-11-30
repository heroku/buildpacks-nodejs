use heroku_nodejs_utils::{package_json::PackageJson, vrs::Requirement};
use std::path::Path;

// Reads parsed `engines.yarn` requirement from a `PackageJson`.
pub(crate) fn requested_yarn_range(pkg_json: &PackageJson) -> Option<Requirement> {
    pkg_json
        .engines
        .as_ref()
        .and_then(|engines| engines.yarn.clone())
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
    if scripts.is_empty() {
        None
    } else {
        Some(scripts)
    }
}

// Determines if a given `PackageJson` has a start script defined
pub(crate) fn has_start_script(pkg_json: &PackageJson) -> bool {
    pkg_json
        .scripts
        .as_ref()
        .map_or(false, |scripts| scripts.start.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;
    use heroku_nodejs_utils::package_json::Scripts;

    #[test]
    fn test_get_build_scripts_all() {
        let pkg_json = PackageJson {
            scripts: Some(Scripts {
                build: Some("echo 'build'".to_owned()),
                heroku_prebuild: Some("echo 'heroku-prebuild'".to_owned()),
                heroku_build: Some("echo 'build'".to_owned()),
                heroku_postbuild: Some("echo 'heroku-postbuild'".to_owned()),
                ..Scripts::default()
            }),
            ..PackageJson::default()
        };
        let build_scripts = get_build_scripts(&pkg_json).expect("Expected build scripts");

        assert_eq!("heroku-prebuild", build_scripts[0]);
        assert_eq!("heroku-build", build_scripts[1]);
        assert_eq!("heroku-postbuild", build_scripts[2]);
    }

    #[test]
    fn test_get_build_scripts_build_only() {
        let pkg_json = PackageJson {
            scripts: Some(Scripts {
                build: Some("echo 'build'".to_owned()),
                ..Scripts::default()
            }),
            ..PackageJson::default()
        };
        let build_scripts = get_build_scripts(&pkg_json).expect("Expected build scripts");

        assert_eq!("build", build_scripts[0]);
    }

    #[test]
    fn test_cache_populated() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test/fixtures/yarn-3-modules-zero/.yarn/cache");
        assert!(
            cache_populated(&path),
            "Expected zero-install app to have a populated cache"
        );
    }

    #[test]
    fn test_cache_unpopulated() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test/fixtures/yarn-3-pnp-nonzero/.yarn/cache");
        assert!(
            !cache_populated(&path),
            "Expected non-zero-install app to have an unpopulated cache"
        );
    }
}
