use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::{CmdError, CommandWithName};
use libcnb::Env;
use std::io::Stderr;
use std::{path::Path, process::Command};

/// Execute `pnpm install` to install dependencies for a pnpm project.
pub(crate) fn pnpm_install(
    pnpm_env: &Env,
    mut log: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, CmdError> {
    log.stream_cmd(
        Command::new("pnpm")
            .args(["install", "--frozen-lockfile"])
            .envs(pnpm_env),
    )?;
    Ok(log)
}

/// Execute `pnpm run` commands like `build`.
pub(crate) fn pnpm_run(
    pnpm_env: &Env,
    script: &str,
    mut log: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, CmdError> {
    log.stream_with(
        format!("Running {script} script", script = style::value(script)),
        |stdout, stderr| {
            Command::new("pnpm")
                .arg("run")
                .arg(script)
                .envs(pnpm_env)
                .stream_output(stdout, stderr)
        },
    )?;
    Ok(log)
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
pub(crate) fn pnpm_store_prune(
    pnpm_env: &Env,
    mut log: Print<SubBullet<Stderr>>,
) -> Result<Print<SubBullet<Stderr>>, CmdError> {
    log.stream_cmd(Command::new("pnpm").args(["store", "prune"]).envs(pnpm_env))?;
    Ok(log)
}
