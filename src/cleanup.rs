use crate::BuildpackBuildContext;
use crate::package_managers::yarn;
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
                CleanupTask::YarnGlobalFolder(global_folder) => {
                    match remove_yarn_global_folder_data(&global_folder) {
                        Ok(true) => task_statuses
                            .push("Removed temporary Yarn data from the build cache".into()),
                        Ok(false) => {}
                        Err(e) => {
                            task_statuses.push(format!("Error during cleanup: {e}"));
                        }
                    }
                }
                CleanupTask::YarnCache {
                    app_dir,
                    cache_dir,
                    mirror_cache_dir,
                    omit,
                } => {
                    // Mirror archives are stale when they're missing from the cache folder, so the
                    // mirror is pruned before archives are removed from the cache folder.
                    if let Some(mirror_cache_dir) = &mirror_cache_dir {
                        match remove_stale_yarn_mirror_archives(
                            &app_dir,
                            mirror_cache_dir,
                            &cache_dir,
                        ) {
                            Ok(0) => {}
                            Ok(removed) => task_statuses.push(format!(
                                "Removed {removed} unused package {} from the Yarn mirror",
                                if removed == 1 { "archive" } else { "archives" }
                            )),
                            Err(e) => {
                                task_statuses.push(format!("Error during cleanup: {e}"));
                            }
                        }
                    }
                    match omit {
                        Some(Omit::CacheDirContents) => {}
                        Some(Omit::ArchivesUsedByPnp) => {
                            task_statuses.extend(yarn_cache_cleanup_status(
                                remove_unused_pnp_cache_archives(&app_dir, &cache_dir),
                            ));
                        }
                        None => task_statuses.extend(yarn_cache_cleanup_status(
                            remove_all_yarn_cache_archives(&cache_dir)
                                .map(PnpCacheCleanup::Removed),
                        )),
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

/// Remove everything from Yarn's global folder except the mirror (`cache/`)
///
/// Yarn also stores data such as npm metadata and linker stores in the global folder. These used
/// to be discarded with `$HOME` each build, but the global folder is now in a cached layer.
fn remove_yarn_global_folder_data(global_folder: &Path) -> std::io::Result<bool> {
    let mut removed = false;
    if !global_folder.exists() {
        return Ok(removed);
    }
    for entry in fs::read_dir(global_folder)? {
        let entry = entry?;
        if entry.file_name() == "cache" {
            continue;
        }
        if entry.file_type()?.is_dir() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
        removed = true;
    }
    Ok(removed)
}

/// Removes package archives from the mirror that the app no longer depends on. Yarn never removes
/// anything from the mirror so, without this, it would grow with every dependency change.
///
/// After install, the cache folder holds an archive for every package in the lockfile, so any
/// mirror archive without a matching cache folder archive is stale.
fn remove_stale_yarn_mirror_archives(
    app_dir: &Path,
    mirror_cache_dir: &Path,
    cache_dir: &Path,
) -> std::io::Result<usize> {
    if !mirror_cache_dir.exists() {
        return Ok(0);
    }
    // A lockfile with a custom name isn't found, so every archive of a used package is kept
    let lockfile = match fs_err::read_to_string(app_dir.join("yarn.lock")) {
        Ok(lockfile) => Some(lockfile),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };
    let cache_key = lockfile.as_deref().and_then(yarn::get_lockfile_cache_key);
    let cache_archives = yarn::list_package_archives(cache_dir)?;
    let mirror_archives = yarn::list_package_archives(mirror_cache_dir)?;
    let stale_archives =
        yarn::get_stale_mirror_archives(&cache_archives, &mirror_archives, cache_key.as_deref());
    for archive in &stale_archives {
        fs_err::remove_file(mirror_cache_dir.join(archive))?;
    }
    Ok(stale_archives.len())
}

fn yarn_cache_cleanup_status(cleanup: std::io::Result<PnpCacheCleanup>) -> Option<String> {
    match cleanup {
        Ok(PnpCacheCleanup::Removed(0)) => None,
        Ok(PnpCacheCleanup::Removed(removed)) => Some(format!(
            "Removed {removed} package {} from the Yarn cache that {} needed at runtime",
            if removed == 1 { "archive" } else { "archives" },
            if removed == 1 { "isn't" } else { "aren't" }
        )),
        Ok(PnpCacheCleanup::Skipped(reason)) => Some(format!(
            "Kept every package archive in the Yarn cache because the archives used by Plug'n'Play couldn't be determined: {reason}"
        )),
        Err(e) => Some(format!("Error during cleanup: {e}")),
    }
}

enum PnpCacheCleanup {
    Removed(usize),
    /// Every archive was kept because the ones Plug'n'Play uses couldn't be determined.
    Skipped(String),
}

/// The cache folder holds an archive for every package Yarn installed, but Plug'n'Play only loads
/// some of them at launch. Archives aren't needed for pruned dev dependencies, unplugged packages,
/// or the originals of packages patched by Yarn.
fn remove_unused_pnp_cache_archives(
    app_dir: &Path,
    cache_dir: &Path,
) -> std::io::Result<PnpCacheCleanup> {
    let pnp_data = yarn::read_pnp_data(app_dir)?;
    let used_archives = match yarn::get_pnp_package_archives(&pnp_data, cache_dir) {
        Ok(used_archives) => used_archives,
        Err(reason) => return Ok(PnpCacheCleanup::Skipped(reason)),
    };

    let mut removed = 0;
    for entry in fs_err::read_dir(cache_dir)? {
        let path = entry?.path();
        let is_unused_archive = path.extension().is_some_and(|ext| ext == "zip")
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| !used_archives.contains(name));
        if is_unused_archive {
            fs_err::remove_file(&path)?;
            removed += 1;
        }
    }
    Ok(PnpCacheCleanup::Removed(removed))
}

/// Packages are installed into `node_modules`, so none of the archives in the cache folder are
/// needed at runtime.
fn remove_all_yarn_cache_archives(cache_dir: &Path) -> std::io::Result<usize> {
    let archives = yarn::list_package_archives(cache_dir)?;
    for archive in &archives {
        fs_err::remove_file(cache_dir.join(archive))?;
    }
    Ok(archives.len())
}

#[derive(Debug, Clone)]
pub(crate) enum CleanupTask {
    /// location contains native module builds with Makefiles
    NodeGypMakefiles(NodeGypArtifactLocation),
    /// pnpm `node_modules/.modules.yaml` (contains non-deterministic `prunedAt` timestamp)
    PnpmModulesYaml(PathBuf),
    /// Yarn global folder (contains the mirror and data that shouldn't accumulate in the cache)
    YarnGlobalFolder(PathBuf),
    /// Yarn cache folder, and the mirror in the build cache it's populated from (committed caches
    /// have no mirror). The mirror keeps archives of packages the app no longer uses.
    YarnCache {
        app_dir: PathBuf,
        cache_dir: PathBuf,
        mirror_cache_dir: Option<PathBuf>,
        /// Cache folder contents left in place; every archive is removed when `None`
        omit: Option<Omit>,
    },
}

/// Contents of a Yarn cache folder left in place by cleanup
#[derive(Debug, Clone, Copy)]
pub(crate) enum Omit {
    /// Archives Plug'n'Play loads at runtime
    ArchivesUsedByPnp,
    /// Everything in the cache folder
    CacheDirContents,
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
    use indoc::indoc;
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
    fn test_remove_yarn_global_folder_data_keeps_mirror() {
        let temp = TempDir::new().unwrap();
        let global_folder = temp.path();
        fs::create_dir_all(global_folder.join("cache")).unwrap();
        fs::write(global_folder.join("cache/pkg-npm-1.0.0-abc-10c0.zip"), "").unwrap();
        fs::create_dir_all(global_folder.join("metadata/npm")).unwrap();
        fs::write(global_folder.join("metadata/npm/pkg.json"), "{}").unwrap();
        fs::write(global_folder.join("telemetry.json"), "{}").unwrap();

        assert!(remove_yarn_global_folder_data(global_folder).unwrap());
        let mut entries = fs::read_dir(global_folder)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        entries.sort();
        assert_eq!(entries, ["cache"]);
        assert!(
            global_folder
                .join("cache/pkg-npm-1.0.0-abc-10c0.zip")
                .exists()
        );

        assert!(!remove_yarn_global_folder_data(global_folder).unwrap());
    }

    #[test]
    fn test_remove_yarn_global_folder_data_missing_folder() {
        let temp = TempDir::new().unwrap();
        assert!(!remove_yarn_global_folder_data(&temp.path().join("missing")).unwrap());
    }

    /// Writes Plug'n'Play data that references `used-npm-1.0.0-abc.zip` and returns the app dir.
    fn create_pnp_app() -> TempDir {
        let app_dir = TempDir::new().unwrap();
        fs::write(
            app_dir.path().join(".pnp.cjs"),
            indoc! {r#"
                "packageLocation": "./"
                "packageLocation": "../yarn_cache/used-npm-1.0.0-abc.zip/node_modules/used/"
            "#},
        )
        .unwrap();
        app_dir
    }

    fn create_cache_dir(files: &[&str]) -> TempDir {
        let cache_dir = TempDir::new().unwrap();
        for file in files {
            fs::write(cache_dir.path().join(file), "").unwrap();
        }
        cache_dir
    }

    fn list_dir(dir: &Path) -> Vec<String> {
        let mut entries = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }

    #[test]
    fn test_remove_unused_pnp_cache_archives_removes_only_unused_archives() {
        let app_dir = create_pnp_app();
        let cache_dir = create_cache_dir(&[
            "used-npm-1.0.0-abc.zip",
            "unused-npm-1.0.0-def.zip",
            ".gitignore",
        ]);
        assert!(matches!(
            remove_unused_pnp_cache_archives(app_dir.path(), cache_dir.path()).unwrap(),
            PnpCacheCleanup::Removed(1)
        ));
        assert_eq!(
            list_dir(cache_dir.path()),
            [".gitignore", "used-npm-1.0.0-abc.zip"]
        );
    }

    #[test]
    fn test_remove_unused_pnp_cache_archives_keeps_every_archive_when_untrusted() {
        let app_dir = create_pnp_app();
        // The referenced archive is missing, so the Plug'n'Play data can't be trusted
        let cache_dir = create_cache_dir(&["unused-npm-1.0.0-def.zip"]);
        assert!(matches!(
            remove_unused_pnp_cache_archives(app_dir.path(), cache_dir.path()).unwrap(),
            PnpCacheCleanup::Skipped(_)
        ));
        assert_eq!(list_dir(cache_dir.path()), ["unused-npm-1.0.0-def.zip"]);
    }

    #[test]
    fn test_remove_unused_pnp_cache_archives_missing_pnp_data() {
        let app_dir = TempDir::new().unwrap();
        let cache_dir = create_cache_dir(&["unused-npm-1.0.0-def.zip"]);
        assert!(remove_unused_pnp_cache_archives(app_dir.path(), cache_dir.path()).is_err());
        assert_eq!(list_dir(cache_dir.path()), ["unused-npm-1.0.0-def.zip"]);
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
