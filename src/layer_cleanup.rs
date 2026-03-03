use bullet_stream::global::print;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub(crate) enum LayerKind {
    /// pnpm virtual store layer (contains native module builds with Makefiles)
    PnpmVirtualStore,
    /// App `node_modules` directory (contains native module builds with Makefiles)
    App,
}

#[derive(Debug, Clone)]
pub(crate) struct LayerCleanupTarget {
    pub(crate) path: PathBuf,
    pub(crate) kind: LayerKind,
}

/// Remove Makefile files from native module build directories
///
/// These files have non-deterministic dependency ordering causing layer invalidation.
/// See: <https://github.com/nodejs/node-gyp/issues/3061>
///
/// This is fixed in node-gyp [11.3.0](https://github.com/nodejs/node-gyp/releases/tag/v11.3.0) which
/// includes changes from <https://github.com/nodejs/gyp-next/releases/tag/v0.20.1> that close the above
/// issue. Support availability in tooling follows:
///
/// | Tool    | Version Range | Notes                                                                          |
/// |---------|---------------|--------------------------------------------------------------------------------|
/// | Node.js | >= 24.10.0    | Via bundled npm v11.6.1                                                        |
/// | npm     | >= 11.6.1     |                                                                                |
/// | pnpm    | >= 10.29.0    |                                                                                |
/// | Yarn    | N/A           | Yarn does not bundle node-gyp but relies on node-gyp being provided by Node.js |
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

    let removed = remove_build_makefiles(path)?;

    let from = match target.kind {
        LayerKind::PnpmVirtualStore => "pnpm virtual store layer",
        LayerKind::App => "the application's node_modules directory",
    };

    if removed > 0 {
        print::sub_bullet(format!(
            "Removed {removed} Makefile {} from {from}",
            if removed <= 1 {
                "artifact"
            } else {
                "artifacts"
            }
        ));
    } else {
        print::sub_bullet(format!("Nothing to remove from {from}"));
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
