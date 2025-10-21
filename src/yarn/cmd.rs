use crate::utils::vrs::{Version, VersionError};
use bullet_stream::global::print;
use fun_run::{CmdError, CommandWithName};
use libcnb::Env;
use std::{path::Path, process::Command};

#[derive(Debug)]
pub(crate) enum YarnVersionError {
    Command(CmdError),
    Parse(String, VersionError),
}

/// Execute `yarn --version` to determine what version of `yarn` is in effect
/// for this codebase.
pub(crate) fn yarn_version(env: &Env) -> Result<Version, YarnVersionError> {
    Command::new("yarn")
        .arg("--version")
        .envs(env)
        .named_output()
        .map_err(YarnVersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| YarnVersionError::Parse(stdout, e))
        })
}

pub(crate) fn yarn_prune(env: &Env) -> Result<(), CmdError> {
    print::sub_stream_cmd(
        Command::new("yarn")
            .args([
                "install",
                "--production",
                "--frozen-lockfile",
                "--ignore-engines",
                "--ignore-scripts",
                "--prefer-offline",
            ])
            .envs(env),
    )
    .map(|_| ())
}

pub(crate) fn yarn_prune_with_plugin(env: &Env, plugin_path: &Path) -> Result<(), CmdError> {
    print::sub_stream_cmd(
        Command::new("yarn")
            .envs(env)
            .env("YARN_PLUGINS", plugin_path)
            .args(["heroku", "prune"]),
    )
    .map(|_| ())
}
