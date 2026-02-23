use bullet_stream::global::print;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub(crate) enum LayerKind {
    /// pnpm virtual store layer (contains native module builds with Makefiles)
    Virtual,
}

#[derive(Debug, Clone)]
pub(crate) struct LayerCleanupTarget {
    pub(crate) path: PathBuf,
    pub(crate) kind: LayerKind,
}

/// Remove Makefile files from native module build directories
/// These files have non-deterministic dependency ordering causing layer invalidation
fn remove_build_makefiles(base_path: &Path) -> std::io::Result<usize> {
    let makefile_dir_entries = WalkDir::new(base_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|dir_entry| {
            dir_entry.file_type().is_file() && dir_entry.path().ends_with("build/Makefile")
        });

    let mut removed_count = 0;
    for entry in makefile_dir_entries {
        fs::remove_file(entry.path())?;
        removed_count += 1;
    }

    Ok(removed_count)
}

/// Clean up non-deterministic build artifacts from a layer
pub(crate) fn cleanup_layer(target: &LayerCleanupTarget) -> Result<(), std::io::Error> {
    let path = &target.path;

    if !path.exists() {
        // Layer doesn't exist, nothing to clean
        return Ok(());
    }

    match target.kind {
        LayerKind::Virtual => {
            // pnpm virtual store: contains symlinked packages with native module builds
            // Clean Makefiles from: virtual/store/*/node_modules/*/build/
            print::bullet("Cleaning up pnpm virtual store layer");
            let removed = remove_build_makefiles(path)?;
            if removed > 0 {
                print::sub_bullet(format!("Removed {removed} Makefile artifacts"));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_remove_build_makefiles() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create build directory with Makefile
        let build_dir = base.join("node_modules/some-package/build");
        fs::create_dir_all(&build_dir).unwrap();
        fs::write(build_dir.join("Makefile"), b"makefile content").unwrap();
        fs::write(build_dir.join("output.o"), b"binary").unwrap(); // Should not be removed

        let removed = remove_build_makefiles(base).unwrap();

        assert_eq!(removed, 1); // Makefile
        assert!(!build_dir.join("Makefile").exists());
        assert!(build_dir.join("output.o").exists()); // Not a makefile
    }
}
