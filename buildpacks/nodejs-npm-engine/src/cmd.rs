use heroku_nodejs_utils::vrs::Version;
use libcnb::Env;
use std::process::{Command, ExitStatus};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    Wait(std::io::Error),
    Exit(ExitStatus),
    Parse(String),
}

pub(crate) fn node_version(env: &Env) -> Result<Version> {
    exec_version_command("node", env)
}

pub(crate) fn npm_version(env: &Env) -> Result<Version> {
    exec_version_command("npm", env)
}

fn exec_version_command(program: &str, env: &Env) -> Result<Version> {
    let output = Command::new(program)
        .args(["--version"])
        .envs(env)
        .output()
        .map_err(Error::Wait)?;

    if !output.status.success() {
        return Err(Error::Exit(output.status));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .parse::<Version>()
        .map_err(|_| Error::Parse(stdout.into_owned()))
}
