use std::path::Path;

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
            .join("./tests/fixtures/yarn-3-modules-zero/.yarn/cache");
        assert!(
            cache_populated(&path),
            "Expected zero-install app to have a populated cache"
        );
    }

    #[test]
    fn test_cache_unpopulated() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("./tests/fixtures/yarn-4-pnp-nonzero/.yarn/cache");
        assert!(
            !cache_populated(&path),
            "Expected non-zero-install app to have an unpopulated cache"
        );
    }
}
