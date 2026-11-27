//! Regenerates the Plug'n'Play data fixtures, with the `expected-archives.txt` next to each of
//! them, and the Yarn mirror fixtures in `tests/fixtures/yarn-data`. See its README.

use fs_err as fs;
use indoc::formatdoc;
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

const LIST_PNP_ARCHIVES_SCRIPT: &str = include_str!("../list-pnp-archives.cjs");
const RUNTIME_OMITTED: &str =
    "// The rest of the Plug'n'Play runtime is omitted from this fixture.\n";

struct Fixture {
    name: &'static str,
    yarn_version: &'static str,
    project: &'static str,
    pnp_inlining: bool,
    /// Yarn 2 names the Plug'n'Play runtime `.pnp.js`.
    pnp_file: &'static str,
}

const FIXTURES: [Fixture; 6] = [
    Fixture {
        name: "yarn-2.4.1",
        yarn_version: "2.4.1",
        project: "pnp-project",
        pnp_inlining: true,
        pnp_file: ".pnp.js",
    },
    Fixture {
        name: "yarn-3.8.7",
        yarn_version: "3.8.7",
        project: "pnp-project",
        pnp_inlining: true,
        pnp_file: ".pnp.cjs",
    },
    Fixture {
        name: "yarn-4.19.0",
        yarn_version: "4.19.0",
        project: "pnp-project",
        pnp_inlining: true,
        pnp_file: ".pnp.cjs",
    },
    Fixture {
        name: "yarn-4.19.0-no-inlining",
        yarn_version: "4.19.0",
        project: "pnp-project",
        pnp_inlining: false,
        pnp_file: ".pnp.cjs",
    },
    Fixture {
        name: "yarn-4.19.0-unplugged-only",
        yarn_version: "4.19.0",
        project: "unplugged-only",
        pnp_inlining: true,
        pnp_file: ".pnp.cjs",
    },
    Fixture {
        name: "yarn-4.19.0-no-dependencies",
        yarn_version: "4.19.0",
        project: "no-dependencies",
        pnp_inlining: true,
        pnp_file: ".pnp.cjs",
    },
];

/// How the app changes between the build that fills the mirror and the build that is pruned.
#[derive(Clone, Copy)]
enum MirrorChange {
    /// `yarn remove` a production dependency that has a package patched by Yarn among its
    /// dependencies, and a dev dependency.
    RemoveDependencies,
    /// Change `compressionLevel`, which changes the cache key, and update the lockfile.
    ChangeCacheKey,
}

struct MirrorFixture {
    name: &'static str,
    yarn_version: &'static str,
    change: MirrorChange,
}

const MIRROR_FIXTURES: [MirrorFixture; 6] = [
    MirrorFixture {
        name: "yarn-2.4.1-dependencies-removed",
        yarn_version: "2.4.1",
        change: MirrorChange::RemoveDependencies,
    },
    MirrorFixture {
        name: "yarn-2.4.1-cache-key-changed",
        yarn_version: "2.4.1",
        change: MirrorChange::ChangeCacheKey,
    },
    MirrorFixture {
        name: "yarn-3.8.7-dependencies-removed",
        yarn_version: "3.8.7",
        change: MirrorChange::RemoveDependencies,
    },
    MirrorFixture {
        name: "yarn-3.8.7-cache-key-changed",
        yarn_version: "3.8.7",
        change: MirrorChange::ChangeCacheKey,
    },
    MirrorFixture {
        name: "yarn-4.19.0-dependencies-removed",
        yarn_version: "4.19.0",
        change: MirrorChange::RemoveDependencies,
    },
    MirrorFixture {
        name: "yarn-4.19.0-cache-key-changed",
        yarn_version: "4.19.0",
        change: MirrorChange::ChangeCacheKey,
    },
];

/// The project the mirror fixtures are generated from.
const MIRROR_PROJECT: &str = "pnp-project";

/// Relative to the repository root.
const FIXTURES_DIR: &str = "tests/fixtures/yarn-data";

fn main() {
    check_required_tools();

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let projects_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("projects");
    generate_pnp_fixtures(&repo_root, &projects_dir);
    generate_mirror_fixtures(&repo_root, &projects_dir);
}

fn generate_pnp_fixtures(repo_root: &Path, projects_dir: &Path) {
    let fixtures_dir = repo_root.join(FIXTURES_DIR);
    let prune_plugin = fs::canonicalize(
        repo_root.join("src/package_managers/@yarnpkg/plugin-prune-dev-dependencies.js"),
    )
    .expect("Should find the buildpack's prune plugin");

    for fixture in &FIXTURES {
        eprintln!(
            "Generating {}",
            Path::new(FIXTURES_DIR).join(fixture.name).display()
        );
        let output_dir = fixtures_dir.join(fixture.name);
        let workspace = tempfile::tempdir().expect("Should create a temporary directory");
        // Mirror the buildpack: the Yarn cache is a sibling of the app, outside the project.
        let app_dir = workspace.path().join("app");
        copy_dir(&projects_dir.join(fixture.project), &app_dir);
        fs::write(
            app_dir.join("list-pnp-archives.cjs"),
            LIST_PNP_ARCHIVES_SCRIPT,
        )
        .expect("Should write the archive listing script");
        write_yarnrc(&app_dir, fixture.pnp_inlining);

        let yarn = format!("yarn@{}", fixture.yarn_version);
        run_yarn(&yarn, &app_dir, None, ["install"]);
        run_yarn(&yarn, &app_dir, Some(&prune_plugin), ["heroku", "prune"]);
        let archives = run_yarn(&yarn, &app_dir, None, ["node", "list-pnp-archives.cjs"]);

        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).expect("Should remove the old fixture");
        }
        fs::create_dir_all(&output_dir).expect("Should create the fixture directory");
        let pnp = fs::read_to_string(app_dir.join(fixture.pnp_file))
            .expect("Should read the Plug'n'Play file");
        fs::write(
            output_dir.join(fixture.pnp_file),
            truncate_pnp_runtime(&pnp),
        )
        .expect("Should write the Plug'n'Play file");
        if !fixture.pnp_inlining {
            fs::copy(
                app_dir.join(".pnp.data.json"),
                output_dir.join(".pnp.data.json"),
            )
            .expect("Should copy the Plug'n'Play data file");
        }
        fs::write(output_dir.join("expected-archives.txt"), archives)
            .expect("Should write the expected archives");
    }
}

/// Each mirror fixture records what the buildpack sees when it prunes the mirror: a build fills the
/// mirror, the app changes, and the next build restores the mirror and installs into an empty
/// cache folder. The archives that should be pruned are worked out by Yarn rather than by the
/// buildpack's logic: the changed app is installed into an empty mirror, and every archive that
/// install doesn't write is stale.
fn generate_mirror_fixtures(repo_root: &Path, projects_dir: &Path) {
    let fixtures_dir = repo_root.join(FIXTURES_DIR);
    for fixture in &MIRROR_FIXTURES {
        eprintln!(
            "Generating {}",
            Path::new(FIXTURES_DIR).join(fixture.name).display()
        );
        let yarn = format!("yarn@{}", fixture.yarn_version);

        // The first build fills the mirror.
        let build = tempfile::tempdir().expect("Should create a temporary directory");
        let app_dir = build.path().join("app");
        copy_dir(&projects_dir.join(MIRROR_PROJECT), &app_dir);
        write_yarnrc(&app_dir, true);
        run_yarn(&yarn, &app_dir, None, ["install"]);

        // The app is changed in a separate workspace with an empty mirror, so the mirror there
        // only holds what the changed app needs.
        let changed = tempfile::tempdir().expect("Should create a temporary directory");
        let changed_app_dir = changed.path().join("app");
        copy_dir(&projects_dir.join(MIRROR_PROJECT), &changed_app_dir);
        for file in ["package.json", "yarn.lock", ".yarnrc.yml"] {
            fs::copy(app_dir.join(file), changed_app_dir.join(file))
                .expect("Should copy the app file");
        }
        match fixture.change {
            MirrorChange::RemoveDependencies => {
                run_yarn(
                    &yarn,
                    &changed_app_dir,
                    None,
                    ["remove", "resolve", "dotenv"],
                );
            }
            MirrorChange::ChangeCacheKey => {
                // Differs from the default of every Yarn version (`mixed` before Yarn 4, `0` after)
                let yarnrc_path = changed_app_dir.join(".yarnrc.yml");
                let yarnrc = fs::read_to_string(&yarnrc_path).expect("Should read the Yarn config");
                fs::write(&yarnrc_path, format!("{yarnrc}compressionLevel: 9\n"))
                    .expect("Should write the Yarn config");
                run_yarn(&yarn, &changed_app_dir, None, ["install"]);
            }
        }
        let needed_mirror_archives = list_zip_files(&mirror_folder(&changed_app_dir));

        // The next build restores the mirror and installs the changed app into an empty cache
        // folder, like the buildpack's `yarn_cache` layer.
        for file in ["package.json", "yarn.lock", ".yarnrc.yml"] {
            fs::copy(changed_app_dir.join(file), app_dir.join(file))
                .expect("Should copy the changed app file");
        }
        fs::remove_dir_all(cache_folder(&app_dir)).expect("Should empty the cache folder");
        run_yarn(&yarn, &app_dir, None, ["install"]);
        let mirror_archives = list_zip_files(&mirror_folder(&app_dir));
        let cache_archives = list_zip_files(&cache_folder(&app_dir));
        let stale_archives = mirror_archives.difference(&needed_mirror_archives);

        let output_dir = fixtures_dir.join(fixture.name);
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).expect("Should remove the old fixture");
        }
        fs::create_dir_all(&output_dir).expect("Should create the fixture directory");
        write_lines(&output_dir.join("mirror-archives.txt"), &mirror_archives);
        write_lines(&output_dir.join("cache-archives.txt"), &cache_archives);
        write_lines(
            &output_dir.join("expected-stale-archives.txt"),
            stale_archives,
        );
        fs::copy(app_dir.join("yarn.lock"), output_dir.join("yarn.lock"))
            .expect("Should copy the lockfile");
    }
}

/// Configures Yarn like the buildpack: the cache folder is outside the project and, with the
/// global cache disabled, Yarn mirrors packages into the global folder set by [`run_yarn`].
fn write_yarnrc(app_dir: &Path, pnp_inlining: bool) {
    fs::write(
        app_dir.join(".yarnrc.yml"),
        formatdoc! {"
            nodeLinker: pnp
            enableGlobalCache: false
            cacheFolder: ../yarn_cache
            enableTelemetry: false
            pnpEnableInlining: {pnp_inlining}
        "},
    )
    .expect("Should write the Yarn config");
}

fn cache_folder(app_dir: &Path) -> PathBuf {
    app_dir.join("../yarn_cache")
}

fn global_folder(app_dir: &Path) -> PathBuf {
    app_dir.join("../yarn_global")
}

/// Yarn stores the mirror in the `cache` folder of the global folder.
fn mirror_folder(app_dir: &Path) -> PathBuf {
    global_folder(app_dir).join("cache")
}

fn list_zip_files(folder: &Path) -> BTreeSet<String> {
    fs::read_dir(folder)
        .expect("Should read the archive folder")
        .map(|entry| entry.expect("Should read an archive folder entry").path())
        .filter(|path| path.extension() == Some(OsStr::new("zip")))
        .map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .expect("Archive name should be UTF-8")
                .to_string()
        })
        .collect()
}

fn write_lines<'a>(path: &Path, lines: impl IntoIterator<Item = &'a String>) {
    let content = lines
        .into_iter()
        .fold(String::new(), |content, line| content + line + "\n");
    fs::write(path, content).expect("Should write the archive listing");
}

/// Runs a specific Yarn version through Corepack and returns its stdout, exiting if it fails.
fn run_yarn<const N: usize>(
    yarn: &str,
    app_dir: &Path,
    plugin: Option<&Path>,
    args: [&str; N],
) -> String {
    let mut command = Command::new("corepack");
    command
        .arg(yarn)
        .args(args)
        .current_dir(app_dir)
        .env("COREPACK_ENABLE_DOWNLOAD_PROMPT", "0")
        .env("COREPACK_ENABLE_AUTO_PIN", "0")
        .env("YARN_ENABLE_IMMUTABLE_INSTALLS", "false")
        // Keeps the mirror in the workspace instead of the user's global folder
        .env("YARN_GLOBAL_FOLDER", global_folder(app_dir))
        .env_remove("YARN_PLUGINS");
    if let Some(plugin) = plugin {
        command.env("YARN_PLUGINS", plugin);
    }
    let output = command.output().expect("Should run corepack");
    if !output.status.success() {
        eprintln!(
            "{}",
            formatdoc! {"
                `corepack {yarn} {args}` failed: {status}
                {stdout}{stderr}",
                args = args.join(" "),
                status = output.status,
                stdout = String::from_utf8_lossy(&output.stdout),
                stderr = String::from_utf8_lossy(&output.stderr),
            }
        );
        std::process::exit(1);
    }
    String::from_utf8(output.stdout).expect("Yarn output should be UTF-8")
}

/// Yarn itself is run through Corepack, which downloads each Yarn version the fixtures need, so
/// only Node.js and Corepack have to be installed. Exits if either is missing.
fn check_required_tools() {
    let missing = [
        ("node", "Install Node.js from https://nodejs.org/en/download"),
        (
            "corepack",
            "Corepack is bundled with Node.js < 25; for newer versions run `npm install -g corepack`",
        ),
    ]
    .into_iter()
    .filter(|(tool, _)| {
        !Command::new(tool)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
    })
    .map(|(tool, hint)| format!("- `{tool}` wasn't found. {hint}"))
    .collect::<Vec<_>>();
    if !missing.is_empty() {
        eprintln!(
            "{}",
            formatdoc! {"
                Required tools are missing:
                {}
            ", missing.join("\n") }
        );
        std::process::exit(1);
    }
}

/// Keeps everything up to the end of the `$$SETUP_STATE` function, which holds the data, and
/// replaces the Plug'n'Play runtime that follows it.
fn truncate_pnp_runtime(pnp: &str) -> String {
    let start = pnp
        .find("function $$SETUP_STATE")
        .expect("Plug'n'Play file should have a $$SETUP_STATE function");
    let end = pnp[start..]
        .find("\n\n")
        .expect("$$SETUP_STATE function should be followed by a blank line");
    format!("{}\n\n{RUNTIME_OMITTED}", &pnp[..start + end])
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("Should create the project directory");
    for entry in fs::read_dir(from).expect("Should read the project directory") {
        let entry = entry.expect("Should read a project directory entry");
        let target: PathBuf = to.join(entry.file_name());
        if entry
            .file_type()
            .expect("Should get the file type")
            .is_dir()
        {
            copy_dir(&entry.path(), &target);
        } else if entry.path().extension() != Some(OsStr::new("lock")) {
            fs::copy(entry.path(), &target).expect("Should copy a project file");
        }
    }
}
