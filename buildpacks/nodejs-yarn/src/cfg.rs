use heroku_nodejs_utils::{package_json::PackageJson, vrs::Requirement};
use std::path::Path;

/// Reads parsed `engines.yarn` requirement from a `PackageJson`.
pub(crate) fn requested_yarn_range(pkg_json: &PackageJson) -> Option<Requirement> {
    pkg_json
        .engines
        .as_ref()
        .and_then(|engines| engines.yarn.clone())
}

/// A yarn cache is populated if it exists, and has non-hidden files.
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

#[cfg(test)]
mod tests {
    use super::*;

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
