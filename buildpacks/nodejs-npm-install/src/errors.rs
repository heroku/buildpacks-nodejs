use crate::cmd;
use heroku_nodejs_utils::package_json::PackageJsonError;
use libherokubuildpack::log::log_error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub(crate) enum NpmInstallBuildpackError {
    Application(heroku_nodejs_utils::application::Error),
    BuildScript(cmd::Error),
    NpmInstall(cmd::Error),
    NpmSetCacheDir(cmd::Error),
    NpmSetNoAudit(cmd::Error),
    NpmVersion(cmd::Error),
    PackageJson(PackageJsonError),
}

pub(crate) fn log_user_errors(error: NpmInstallBuildpackError) {
    match error {
        NpmInstallBuildpackError::PackageJson(error) => {
            log_error(
                "npm package.json error",
                format!("Couldn't parse package.json: {error}"),
            );
        }

        NpmInstallBuildpackError::NpmSetCacheDir(error) => {
            log_error(
                "npm set cache dir error",
                format!("Couldn't set the npm cache dir: {error}"),
            );
        }

        NpmInstallBuildpackError::NpmSetNoAudit(error) => {
            log_error(
                "npm set audit error",
                format!("Couldn't disable npm auditing: {error}"),
            );
        }

        NpmInstallBuildpackError::NpmVersion(error) => {
            log_error(
                "npm version error",
                format!("Couldn't get the version from npm: {error}"),
            );
        }

        NpmInstallBuildpackError::NpmInstall(error) => {
            log_error(
                "npm install error",
                format!("Couldn't execute npm install: {error}"),
            );
        }

        NpmInstallBuildpackError::BuildScript(error) => {
            log_error(
                "npm run build script error",
                format!("Couldn't run the build script: {error}"),
            );
        }

        NpmInstallBuildpackError::Application(error) => {
            log_error("Application build error", error.to_string())
        }
    }
}

impl Display for cmd::Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            cmd::Error::Spawn(_error) => {
                write!(f, "spawn error")
            }

            cmd::Error::Wait(_error) => {
                write!(f, "wait error")
            }

            cmd::Error::Exit(_error) => {
                write!(f, "exit error")
            }

            cmd::Error::Parse(_error) => {
                write!(f, "parse error")
            }
        }
    }
}
