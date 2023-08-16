use std::fmt::{Display, Formatter};
use std::path::Path;

pub type Result<T> = std::result::Result<T, Error>;

/// Checks for npm, Yarn, pnpm, and shrink-wrap lockfiles and raises an error if multiple are detected.
///
/// # Errors
///
/// Will return `Err` if more than one lockfile is present in the given directory.
pub fn check_for_multiple_lockfiles(app_dir: &Path) -> Result<()> {
    let detected_lockfiles = [
        "npm-shrinkwrap.json",
        "package-lock.json",
        "pnpm-lock.yaml",
        "yarn.lock",
    ]
    .into_iter()
    .filter(|lockfile| app_dir.join(lockfile).exists())
    .map(std::string::ToString::to_string)
    .collect::<Vec<_>>();

    match detected_lockfiles.len() {
        0 | 1 => Ok(()),
        _ => Err(Error::MultipleLockfiles(detected_lockfiles)),
    }
}

/// Checks if the `node_modules` folder is present in the given directory which indicates that
/// the application contains files that it shouldn't in its git repository. If this is the case,
/// a [`Warning`] will be returned.
#[must_use]
pub fn warn_prebuilt_modules(app_dir: &Path) -> Option<Warning> {
    if app_dir.join("node_modules").exists() {
        Some(Warning {
            header: "node_modules checked into source control".to_string(),
            body: "https://devcenter.heroku.com/articles/node-best-practices#only-git-the-important-bits".to_string(),
        })
    } else {
        None
    }
}

pub struct Warning {
    pub header: String,
    pub body: String,
}

#[derive(Debug)]
pub enum Error {
    MultipleLockfiles(Vec<String>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MultipleLockfiles(lockfiles) => {
                write!(
                    f,
                    "Build failed because multiple lockfiles were detected:\n{}",
                    lockfiles
                        .iter()
                        .map(|lockfile| format!("- {lockfile}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }
        }
    }
}
