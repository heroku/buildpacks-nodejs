use bullet_stream::global::print;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub(crate) enum LayerKind {
    /// pnpm virtual store layer (contains native module builds with Makefiles)
    PnpmVirtualStore,
    /// App directory (contains native module builds with Makefiles)
    App,
    /// pnpm `node_modules/.modules.yaml` (contains non-deterministic `prunedAt` timestamp)
    PnpmModulesYaml,
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

/// Remove pnpm's `.modules.yaml` file from `node_modules/`
///
/// This file contains a `prunedAt` timestamp that changes on every install,
/// causing non-deterministic app layer content. pnpm regenerates this file
/// on every install so it is safe to remove.
fn remove_pnpm_modules_yaml(app_dir: &Path) -> std::io::Result<bool> {
    let modules_yaml = app_dir.join("node_modules/.modules.yaml");
    if modules_yaml.exists() {
        fs::remove_file(&modules_yaml)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Clean up non-deterministic build artifacts from a layer
pub(crate) fn cleanup_layer(target: &LayerCleanupTarget) -> Result<(), std::io::Error> {
    let path = &target.path;

    if !path.exists() {
        // Layer doesn't exist, nothing to clean
        return Ok(());
    }

    match target.kind {
        LayerKind::PnpmVirtualStore | LayerKind::App => {
            let removed = remove_build_makefiles(path)?;

            let from = match target.kind {
                LayerKind::PnpmVirtualStore => "pnpm virtual store layer",
                LayerKind::App => "the application directory",
                LayerKind::PnpmModulesYaml => unreachable!(),
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
        }
        LayerKind::PnpmModulesYaml => {
            if remove_pnpm_modules_yaml(path)? {
                print::sub_bullet("Removed non-deterministic .modules.yaml from node_modules");
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
    fn test_remove_pnpm_modules_yaml() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        let node_modules = base.join("node_modules");
        fs::create_dir_all(&node_modules).unwrap();
        fs::write(node_modules.join(".modules.yaml"), b"prunedAt: some-date").unwrap();

        let removed = remove_pnpm_modules_yaml(base).unwrap();

        assert!(removed);
        assert!(!node_modules.join(".modules.yaml").exists());
    }

    #[test]
    fn test_remove_pnpm_modules_yaml_when_missing() {
        let temp = TempDir::new().unwrap();
        let removed = remove_pnpm_modules_yaml(temp.path()).unwrap();
        assert!(!removed);
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
