use crate::package_manager::PackageManager;
use bullet_stream::style;
use indoc::{formatdoc, writedoc};
use std::fmt::{Display, Formatter};
use std::path::Path;

/// Checks for npm, Yarn, pnpm, and shrink-wrap lockfiles and raises an error if multiple are detected.
///
/// # Errors
///
/// Will return an `Err` when:
/// - More than one lockfile exists in the `app_dir`.
/// - No lockfile exists in the `app_dir`.
pub fn check_for_singular_lockfile(app_dir: &Path) -> Result<(), Error> {
    let detected_lockfiles = [
        PackageManager::Npm,
        PackageManager::Pnpm,
        PackageManager::Yarn,
    ]
    .into_iter()
    .filter(|package_manager| app_dir.join(package_manager.lockfile()).exists())
    .collect::<Vec<_>>();

    match detected_lockfiles.len() {
        0 => Err(Error::MissingLockfile),
        1 => Ok(()),
        _ => Err(Error::MultipleLockfiles(detected_lockfiles)),
    }
}

/// Checks if the `node_modules` folder is present in the given directory which indicates that
/// the application contains files that it shouldn't in its git repository. If this is the case,
/// a warning message will be returned.
#[must_use]
pub fn warn_prebuilt_modules(app_dir: &Path) -> Option<String> {
    if app_dir.join("node_modules").exists() {
        Some(formatdoc! {"
            Warning: {node_modules} checked into source control

            Add these files and directories to {gitignore}. See the Dev Center for more info:
            https://devcenter.heroku.com/articles/node-best-practices#only-git-the-important-bits
        ", node_modules = style::value("node_modules"), gitignore = style::value(".gitignore") })
    } else {
        None
    }
}

#[derive(Debug)]
pub enum Error {
    MissingLockfile,
    MultipleLockfiles(Vec<PackageManager>),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::MissingLockfile => {
                writedoc!(
                    f,
                    "
                        Couldn't determine Node.js package manager. Package \
                        manager lockfile not found.

                        A lockfile from a supported package manager is \
                        required to install Node.js dependencies. The \
                        package.json for this project specifies dependencies, \
                        but there isn't a lockfile.

                        To use npm to install dependencies, run {npm_install}. \
                        This command will generate a {npm_lockfile} lockfile.

                        Or, to use yarn to install dependencies, run {yarn_install}. \
                        This command will generate a {yarn_lockfile} lockfile.

                        Or, to use pnpm to install dependencies, run {pnpm_install}. \
                        This command will generate a {pnpm_lockfile} lockfile.

                        Ensure the resulting lockfile is committed to the repository, then try again.
                    ",
                    npm_install = style::value("npm install"),
                    npm_lockfile = style::value(PackageManager::Npm.lockfile().to_string_lossy()),
                    yarn_install = style::value("yarn install"),
                    yarn_lockfile = style::value(PackageManager::Yarn.lockfile().to_string_lossy()),
                    pnpm_install = style::value("pnpm install"),
                    pnpm_lockfile = style::value(PackageManager::Pnpm.lockfile().to_string_lossy()),
                )?;
                Ok(())
            }
            Error::MultipleLockfiles(package_managers) => {
                let lockfiles = package_managers
                    .iter()
                    .map(|f| f.lockfile().to_string_lossy().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                writedoc!(f, "
                    Multiple lockfiles found: {lockfiles}

                    More than one package manager has created lockfiles for this application. Only one \
                    can be used to install dependencies but the buildpack can't determine which when multiple \
                    lockfiles are present.

                ")?;

                for package_manager in PackageManager::iterator() {
                    writedoc!(f, "- To use {} to install your application's dependencies please delete the following lockfiles:\n\n", style::value(package_manager.to_string()))?;
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
                    f, "
                        See the Knowledge Base for more info: https://help.heroku.com/0KU2EM53

                        Once your application has only one lockfile, commit the results to git and retry your build.
                    "
                )?;

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

                More than one package manager has created lockfiles for this application. Only one can be used to install dependencies but the buildpack can't determine which when multiple lockfiles are present.

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

                Once your application has only one lockfile, commit the results to git and retry your build.
            ", npm = style::value("npm"), pnpm = style::value("pnpm"), yarn = style::value("Yarn") }
        );
    }
}
