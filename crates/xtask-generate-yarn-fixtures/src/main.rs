//! Regenerates the Plug'n'Play data fixtures in `tests/fixtures/yarn-data` and the
//! `expected-archives.txt` next to each of them. See that directory's README.

use fs_err as fs;
use indoc::formatdoc;
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

/// Relative to the repository root.
const FIXTURES_DIR: &str = "tests/fixtures/yarn-data";

fn main() {
    check_required_tools();

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let projects_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("projects");
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
        fs::write(
            app_dir.join(".yarnrc.yml"),
            formatdoc! {"
                nodeLinker: pnp
                enableGlobalCache: false
                cacheFolder: ../yarn_cache
                enableTelemetry: false
                pnpEnableInlining: {}
            ", fixture.pnp_inlining },
        )
        .expect("Should write the Yarn config");

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
