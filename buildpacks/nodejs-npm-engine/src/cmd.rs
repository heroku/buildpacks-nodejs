use commons::fun_run::{CmdError, CommandWithName};
use heroku_nodejs_utils::vrs::Version;
use libcnb::Env;
use std::process::Command;

#[derive(Debug)]
pub(crate) enum Error {
    NpmVersionCommand(CmdError),
    ParseNpmVersion(String),
    NodeVersionCommand(CmdError),
    ParseNodeVersion(String),
}

pub(crate) fn node_version(env: &Env) -> Result<Version, Error> {
    exec_version_command("node", env)
        .map_err(Error::NodeVersionCommand)
        .and_then(|output| output.parse().map_err(|_| Error::ParseNpmVersion(output)))
}

pub(crate) fn npm_version(env: &Env) -> Result<Version, Error> {
    exec_version_command("npm", env)
        .map_err(Error::NpmVersionCommand)
        .and_then(|output| output.parse().map_err(|_| Error::ParseNodeVersion(output)))
}

fn exec_version_command(program: &str, env: &Env) -> Result<String, CmdError> {
    Command::new(program)
        .args(["--version"])
        .envs(env)
        .named_output()
        .and_then(|output| output.nonzero_captured())
        .map(|output| output.stdout_lossy())
}
