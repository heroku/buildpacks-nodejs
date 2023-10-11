use crate::package_manager::PackageManager;
use commons::output::fmt;
use commons::output::section_log::log_warning_later;
use commons::output::warn_later::WarnGuard;
use indoc::{formatdoc, writedoc};
use std::fmt::{Display, Formatter};
use std::io::Stdout;
use std::path::Path;

pub type Result<T> = std::result::Result<T, Error>;

/// Checks for npm, Yarn, pnpm, and shrink-wrap lockfiles and raises an error if multiple are detected.
///
/// # Errors
///
/// Will return `Err` if more than one lockfile is present in the given directory.
pub fn check_for_multiple_lockfiles(app_dir: &Path) -> Result<()> {
    let detected_lockfiles = [
        PackageManager::Npm,
        PackageManager::Pnpm,
        PackageManager::Yarn,
    ]
    .into_iter()
    .filter(|package_manager| app_dir.join(package_manager.lockfile()).exists())
    .collect::<Vec<_>>();

    match detected_lockfiles.len() {
        0 | 1 => Ok(()),
        _ => Err(Error::MultipleLockfiles(detected_lockfiles)),
    }
}

/// Checks if the `node_modules` folder is present in the given directory which indicates that
/// the application contains files that it shouldn't in its git repository. If this is the case,
/// a delayed warning will be published to the logger. To ensure the delayed warning is properly
/// displayed it should be used in conjunction with a [`WarnGuard`].
pub fn warn_prebuilt_modules(app_dir: &Path, _warn_later: &WarnGuard<Stdout>) {
    if app_dir.join("node_modules").exists() {
        log_warning_later(formatdoc! {"
            Warning: {node_modules} checked into source control

            Add these files and directories to {gitignore}. See the Dev Center for more info: 
            https://devcenter.heroku.com/articles/node-best-practices#only-git-the-important-bits
        ", node_modules = fmt::value("node_modules"), gitignore = fmt::value(".gitignore") });
    }
}

#[derive(Debug)]
pub enum Error {
    MultipleLockfiles(Vec<PackageManager>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MultipleLockfiles(package_managers) => {
                let lockfiles = package_managers
                    .iter()
                    .map(|f| f.lockfile().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                writedoc!(f, "
                    Multiple lockfiles found: {lockfiles}
    
                    More than one package manager has created lockfiles for this application. Only one \
                    can be used to install dependencies. 

                ")?;

                for package_manager in PackageManager::iterator() {
                    writedoc!(f, "- To use {} to install your application's dependencies please delete the following lockfiles:\n\n", fmt::value(package_manager.to_string()))?;
                    for other_package_manager in PackageManager::iterator() {
                        if package_manager != other_package_manager {
                            let other_lockfile = other_package_manager
                                .lockfile()
                                .to_string_lossy()
                                .to_string();
                            writedoc!(f, "    $ git rm {other_lockfile}\n")?;
                        }
                    }
                    writedoc!(f, "\n")?;
                }
                writedoc!(
                    f,
                    "See the Knowledge Base for more info: https://help.heroku.com/0KU2EM53\n"
                )?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::application::Error;
    use crate::package_manager::PackageManager;
    use commons::output::fmt;
    use indoc::formatdoc;

    #[test]
    fn test_error_output_for_multiple_lockfiles() {
        let error = Error::MultipleLockfiles(vec![
            PackageManager::Npm,
            PackageManager::Pnpm,
            PackageManager::Yarn,
        ]);
        assert_eq!(
            error.to_string(),
            formatdoc! {"
                Multiple lockfiles found: package-lock.json, pnpm-lock.yaml, yarn.lock

                More than one package manager has created lockfiles for this application. Only one can be used to install dependencies. 

                - To use {npm} to install your application's dependencies please delete the following lockfiles:

                    $ git rm pnpm-lock.yaml
                    $ git rm yarn.lock

                - To use {pnpm} to install your application's dependencies please delete the following lockfiles:

                    $ git rm package-lock.json
                    $ git rm yarn.lock
                
                - To use {yarn} to install your application's dependencies please delete the following lockfiles:
                
                    $ git rm package-lock.json
                    $ git rm pnpm-lock.yaml

                See the Knowledge Base for more info: https://help.heroku.com/0KU2EM53
            ", npm = fmt::value("npm"), pnpm = fmt::value("pnpm"), yarn = fmt::value("Yarn") }
        );
    }
}
