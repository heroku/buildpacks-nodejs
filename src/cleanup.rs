use crate::BuildpackBuildContext;
use bullet_stream::global::print;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Clean up non-deterministic build artifacts from a layer
pub(crate) fn run_post_build_cleanup_tasks(context: &BuildpackBuildContext) {
    let cleanup_tasks = context.cleanup_tasks();
    if !cleanup_tasks.is_empty() {
        print::bullet("Removing non-deterministic build artifacts before export");

        let mut task_statuses = vec![];
        for cleanup_task in cleanup_tasks {
            match cleanup_task {
                CleanupTask::NodeGypMakefiles(artifact_location) => {
                    let base_path = match &artifact_location {
                        NodeGypArtifactLocation::AppDir(path)
                        | NodeGypArtifactLocation::PnpmVirtualStore(path) => path,
                    };
                    match remove_build_makefiles(base_path) {
                        Ok(removed) => {
                            if removed > 0 {
                                task_statuses.push(format!(
                                    "Removed {removed} Makefile {} from {artifact_location}",
                                    if removed <= 1 {
                                        "artifact"
                                    } else {
                                        "artifacts"
                                    }
                                ));
                            }
                        }
                        Err(e) => {
                            task_statuses.push(format!("Error during cleanup: {e}"));
                        }
                    }
                }
                CleanupTask::PnpmModulesYaml(base_path) => {
                    match remove_pnpm_modules_yaml(&base_path) {
                        Ok(true) => task_statuses
                            .push("Removed pnpm metadata file node_modules/.modules.yaml".into()),
                        Ok(false) => {}
                        Err(e) => {
                            task_statuses.push(format!("Error during cleanup: {e}"));
                        }
                    }
                }
            }
        }

        if task_statuses.is_empty() {
            print::sub_bullet("Nothing to cleanup");
        } else {
            for task_status in task_statuses {
                print::sub_bullet(task_status);
            }
        }
    }
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
            dir_entry.file_type().is_file()
                && dir_entry.path().ends_with("build/Makefile")
                && dir_entry
                    .path()
                    .ancestors()
                    .any(|a| a.file_name().is_some_and(|name| name == "node_modules"))
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

#[derive(Debug, Clone)]
pub(crate) enum CleanupTask {
    /// location contains native module builds with Makefiles
    NodeGypMakefiles(NodeGypArtifactLocation),
    /// pnpm `node_modules/.modules.yaml` (contains non-deterministic `prunedAt` timestamp)
    PnpmModulesYaml(PathBuf),
}

#[derive(Debug, Clone)]
pub(crate) enum NodeGypArtifactLocation {
    AppDir(PathBuf),
    PnpmVirtualStore(PathBuf),
}

impl Display for NodeGypArtifactLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeGypArtifactLocation::AppDir(_) => write!(f, "the application directory"),
            NodeGypArtifactLocation::PnpmVirtualStore(_) => write!(f, "pnpm virtual store layer"),
        }
    }
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

        // Create build directory with Makefile inside node_modules
        let build_dir = base.join("node_modules/some-package/build");
        fs::create_dir_all(&build_dir).unwrap();
        fs::write(build_dir.join("Makefile"), b"makefile content").unwrap();
        fs::write(build_dir.join("output.o"), b"binary").unwrap();

        let removed = remove_build_makefiles(base).unwrap();

        assert_eq!(removed, 1);
        assert!(!build_dir.join("Makefile").exists());
        assert!(build_dir.join("output.o").exists());
    }

    #[test]
    fn test_remove_build_makefiles_ignores_non_node_modules() {
        let temp = TempDir::new().unwrap();
        let base = temp.path();

        // Makefile outside of node_modules should not be removed
        let app_build_dir = base.join("src/native/build");
        fs::create_dir_all(&app_build_dir).unwrap();
        fs::write(app_build_dir.join("Makefile"), b"app makefile").unwrap();

        // Makefile inside node_modules should be removed
        let nm_build_dir = base.join("node_modules/some-package/build");
        fs::create_dir_all(&nm_build_dir).unwrap();
        fs::write(nm_build_dir.join("Makefile"), b"node-gyp makefile").unwrap();

        let removed = remove_build_makefiles(base).unwrap();

        assert_eq!(removed, 1);
        assert!(app_build_dir.join("Makefile").exists()); // Not removed
        assert!(!nm_build_dir.join("Makefile").exists()); // Removed
    }
}
