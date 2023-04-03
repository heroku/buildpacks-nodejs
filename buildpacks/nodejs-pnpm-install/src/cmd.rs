use libcnb::Env;
use std::{path::Path, process::Command};

#[derive(Debug)]
pub(crate) enum Error {
    Spawn(std::io::Error),
    Wait(std::io::Error),
    Exit(std::process::ExitStatus),
}

/// Execute `pnpm install` to install dependencies for a pnpm project.
pub(crate) fn pnpm_install(pnpm_env: &Env) -> Result<(), Error> {
    let mut process = Command::new("pnpm")
        .args(["install", "--frozen-lockfile"])
        .envs(pnpm_env)
        .spawn()
        .map_err(Error::Spawn)?;

    let status = process.wait().map_err(Error::Wait)?;

    status.success().then_some(()).ok_or(Error::Exit(status))
}

/// Execute `pnpm run` commands like `build`.
pub(crate) fn pnpm_run(pnpm_env: &Env, script: &str) -> Result<(), Error> {
    let status = Command::new("pnpm")
        .arg("run")
        .arg(script)
        .envs(pnpm_env)
        .spawn()
        .map_err(Error::Spawn)?
        .wait()
        .map_err(Error::Wait)?;

    status.success().then_some(()).ok_or(Error::Exit(status))
}

/// Execute `pnpm config set store-dir` to set pnpm's addressable store location.
pub(crate) fn pnpm_set_store_dir(pnpm_env: &Env, location: &Path) -> Result<(), Error> {
    let status = Command::new("pnpm")
        .args(["config", "set", "store-dir", &location.to_string_lossy()])
        .envs(pnpm_env)
        .spawn()
        .map_err(Error::Spawn)?
        .wait()
        .map_err(Error::Wait)?;

    status.success().then_some(()).ok_or(Error::Exit(status))
}

/// Execute `pnpm config set virtual-store-dir` to set pnpm's virtual store location.
pub(crate) fn pnpm_set_virtual_dir(pnpm_env: &Env, location: &Path) -> Result<(), Error> {
    let status = Command::new("pnpm")
        .args([
            "config",
            "set",
            "virtual-store-dir",
            &location.to_string_lossy(),
        ])
        .envs(pnpm_env)
        .spawn()
        .map_err(Error::Spawn)?
        .wait()
        .map_err(Error::Wait)?;

    status.success().then_some(()).ok_or(Error::Exit(status))
}

/// Execute `pnpm store prune` to remove unused dependencies from the
/// content-addressable store.
pub(crate) fn pnpm_store_prune(pnpm_env: &Env) -> Result<(), Error> {
    let status = Command::new("pnpm")
        .args(["store", "prune"])
        .envs(pnpm_env)
        .spawn()
        .map_err(Error::Spawn)?
        .wait()
        .map_err(Error::Wait)?;

    status.success().then_some(()).ok_or(Error::Exit(status))
}
