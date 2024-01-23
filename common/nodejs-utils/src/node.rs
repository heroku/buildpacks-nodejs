use crate::vrs::{Version, VersionError};
use fun_run::{CmdError, CommandWithName, NamedOutput};
use libcnb::Env;
use std::process::Command;

#[derive(Debug)]
pub enum GetNodeVersionError {
    Command(CmdError),
    Parse(String, VersionError),
}

pub fn get_node_version(env: &Env) -> Result<Version, GetNodeVersionError> {
    Command::new("node")
        .arg("--version")
        .envs(env)
        .named_output()
        .and_then(NamedOutput::nonzero_captured)
        .map_err(GetNodeVersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| GetNodeVersionError::Parse(stdout, e))
        })
}
