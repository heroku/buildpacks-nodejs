use super::main::PnpmInstallBuildpackError;
use crate::BuildpackError;
use crate::utils::vrs::{Version, VersionError};
use bullet_stream::global::print;
use fun_run::{CmdError, CommandWithName};
use libcnb::Env;
use std::process::Command;

/// Execute `pnpm run` commands like `build`.
pub(crate) fn pnpm_run(pnpm_env: &Env, script: &str) -> Result<(), CmdError> {
    print::sub_stream_cmd(Command::new("pnpm").arg("run").arg(script).envs(pnpm_env)).map(|_| ())
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

impl From<PnpmVersionError> for BuildpackError {
    fn from(value: PnpmVersionError) -> Self {
        PnpmInstallBuildpackError::PnpmVersion(value).into()
    }
}
