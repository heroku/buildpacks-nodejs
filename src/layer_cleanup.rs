use bullet_stream::global::print;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub(crate) enum LayerKind {
    /// Node.js distribution layer (contains npm with node-gyp)
    Dist,
    /// Custom npm version layer (contains npm with node-gyp)
    NpmEngine,
    /// pnpm distribution layer (contains pnpm with node-gyp)
    Pnpm,
    /// pnpm virtual store layer (contains native module builds with Makefiles)
    Virtual,
}

#[derive(Debug, Clone)]
pub(crate) struct LayerCleanupTarget {
    pub(crate) path: PathBuf,
    pub(crate) kind: LayerKind,
}

/// Remove Python bytecode cache files (__pycache__/*.pyc) from node-gyp directories
/// These files are generated during native module compilation and are non-deterministic
fn remove_python_bytecode_cache(base_path: &Path) -> Result<usize, std::io::Error> {
    let mut removed_count = 0;

    // Look for node-gyp/gyp/pylib directory containing Python source
    let pylib_path = base_path.join("node_modules/node-gyp/gyp/pylib");

    if !pylib_path.exists() {
        return Ok(0);
    }

    // Walk the pylib tree looking for __pycache__ directories
    for entry in WalkDir::new(&pylib_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir() && e.file_name() == "__pycache__")
    {
        // Remove the entire __pycache__ directory
        fs::remove_dir_all(entry.path())?;
        removed_count += 1;
    }

    Ok(removed_count)
}

/// Remove Makefile and *.mk files from native module build directories
/// These files have non-deterministic dependency ordering causing layer invalidation
fn remove_build_makefiles(base_path: &Path) -> Result<usize, std::io::Error> {
    let mut removed_count = 0;

    // Walk directory tree looking for build/Makefile patterns
    for entry in WalkDir::new(base_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            if !e.file_type().is_file() {
                return false;
            }

            // Check if this is a Makefile or .mk file in a build/ directory
            let path = e.path();
            if let Some(parent) = path.parent()
                && parent.file_name() == Some(std::ffi::OsStr::new("build"))
                && let Some(filename) = path.file_name()
            {
                return filename.to_string_lossy() == "Makefile";
            }

            false
        })
    {
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
        LayerKind::Dist => {
            // npm dist layer: contains bundled npm with node-gyp
            // Clean Python bytecode from: dist/lib/node_modules/npm/node_modules/node-gyp/
            print::bullet("Cleaning up Node.js installation layer");
            let npm_modules = path.join("lib/node_modules/npm");
            if npm_modules.exists() {
                let removed = remove_python_bytecode_cache(&npm_modules)?;
                if removed > 0 {
                    print::sub_bullet(format!("Removed {removed} __pycache__ directories"));
                }
            }
        }
        LayerKind::NpmEngine => {
            // npm_engine layer: custom npm version installed via npm registry
            // Clean Python bytecode from: npm_engine/node_modules/node-gyp/
            print::bullet("Cleaning up npm installation layer");
            let removed = remove_python_bytecode_cache(path)?;
            if removed > 0 {
                print::sub_bullet(format!("Removed {removed} __pycache__ directories"));
            }
        }
        LayerKind::Pnpm => {
            // pnpm layer: contains pnpm distribution with node-gyp
            // Clean Python bytecode from: pnpm/dist/node_modules/node-gyp/
            print::bullet("Cleaning up pnpm installation layer");
            let pnpm_dist = path.join("dist");
            if pnpm_dist.exists() {
                let removed = remove_python_bytecode_cache(&pnpm_dist)?;
                if removed > 0 {
                    print::sub_bullet(format!("Removed {removed} __pycache__ directories"));
                }
            }
        }
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
    fn test_remove_python_bytecode_cache() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Create directory structure with __pycache__
        let pycache_path = base.join("node_modules/node-gyp/gyp/pylib/gyp/__pycache__");
        fs::create_dir_all(&pycache_path).unwrap();
        fs::write(pycache_path.join("test.cpython-312.pyc"), b"bytecode").unwrap();

        let removed = remove_python_bytecode_cache(base).unwrap();

        assert_eq!(removed, 1);
        assert!(!pycache_path.exists());
    }

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
