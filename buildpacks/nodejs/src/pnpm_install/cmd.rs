use super::PnpmInstallBuildpackError;
use bullet_stream::global::print;
use fun_run::{CmdError, CommandWithName};
use heroku_nodejs_utils::vrs::{Version, VersionError};
use libcnb::Env;
use std::{path::Path, process::Command};

/// Execute `pnpm install` to install dependencies for a pnpm project.
pub(crate) fn pnpm_install(pnpm_env: &Env) -> Result<(), CmdError> {
    print::sub_stream_cmd(
        Command::new("pnpm")
            .args(["install", "--frozen-lockfile"])
            .envs(pnpm_env),
    )
    .map(|_| ())
}

/// Execute `pnpm run` commands like `build`.
pub(crate) fn pnpm_run(pnpm_env: &Env, script: &str) -> Result<(), CmdError> {
    print::sub_stream_cmd(Command::new("pnpm").arg("run").arg(script).envs(pnpm_env)).map(|_| ())
}

/// Execute `pnpm config set store-dir` to set pnpm's addressable store location.
pub(crate) fn pnpm_set_store_dir(pnpm_env: &Env, location: &Path) -> Result<(), CmdError> {
    Command::new("pnpm")
        .args(["config", "set", "store-dir", &location.to_string_lossy()])
        .envs(pnpm_env)
        .named_output()
        .map(|_| ())
}

/// Execute `pnpm config set virtual-store-dir` to set pnpm's virtual store location.
pub(crate) fn pnpm_set_virtual_dir(pnpm_env: &Env, location: &Path) -> Result<(), CmdError> {
    Command::new("pnpm")
        .args([
            "config",
            "set",
            "virtual-store-dir",
            &location.to_string_lossy(),
        ])
        .envs(pnpm_env)
        .named_output()
        .map(|_| ())
}

/// Execute `pnpm store prune` to remove unused dependencies from the
/// content-addressable store.
pub(crate) fn pnpm_store_prune(pnpm_env: &Env) -> Result<(), CmdError> {
    print::sub_stream_cmd(Command::new("pnpm").args(["store", "prune"]).envs(pnpm_env)).map(|_| ())
}

pub(crate) fn pnpm_prune_dev_dependencies(
    env: &Env,
    extra_args: Vec<&str>,
) -> Result<(), CmdError> {
    print::sub_stream_cmd(
        Command::new("pnpm")
            .args(["prune", "--prod"])
            .args(extra_args)
            .envs(env),
    )
    .map(|_| ())
}

pub(crate) fn pnpm_version(env: &Env) -> Result<Version, PnpmVersionError> {
    Command::new("pnpm")
        .arg("--version")
        .envs(env)
        .named_output()
        .map_err(PnpmVersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse()
                .map_err(|e| PnpmVersionError::Parse(stdout, e))
        })
}

#[derive(Debug)]
pub(crate) enum PnpmVersionError {
    Command(CmdError),
    Parse(String, VersionError),
}

impl From<PnpmVersionError> for libcnb::Error<PnpmInstallBuildpackError> {
    fn from(value: PnpmVersionError) -> Self {
        libcnb::Error::BuildpackError(PnpmInstallBuildpackError::PnpmVersion(value))
    }
}
