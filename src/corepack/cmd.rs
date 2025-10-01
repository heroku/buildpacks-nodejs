use crate::utils::vrs::{Version, VersionError};
use bullet_stream::global::print;
use bullet_stream::style;
use fun_run::CommandWithName;
use libcnb::Env;
use std::{path::Path, process::Command};

#[derive(Debug)]
pub(crate) enum CorepackVersionError {
    Parse(VersionError),
    Command(fun_run::CmdError),
}

/// Execute `corepack --version` to determine corepack version
pub(crate) fn corepack_version(env: &Env) -> Result<Version, CorepackVersionError> {
    let output = Command::new("corepack")
        .arg("--version")
        .envs(env)
        .named_output()
        .map_err(CorepackVersionError::Command)?;

    output
        .stdout_lossy()
        .parse()
        .map_err(CorepackVersionError::Parse)
}

/// Execute `corepack enable` to setup a corepack shim
pub(crate) fn corepack_enable(
    package_manager: &str,
    shim_path: &Path,
    env: &Env,
) -> Result<(), fun_run::CmdError> {
    let shim_path_string = shim_path.to_string_lossy();
    let mut command = Command::new("corepack");
    command.args([
        "enable",
        "--install-directory",
        &shim_path_string,
        package_manager,
    ]);
    command.envs(env);

    print::sub_bullet(format!("Executing {}", style::command(command.name())));

    command.named_output().map(|_| ())
}

/// Execute `corepack prepare` to install the correct package manager
pub(crate) fn corepack_prepare(env: &Env) -> Result<(), fun_run::CmdError> {
    print::sub_stream_cmd(Command::new("corepack").arg("prepare").envs(env)).map(|_| ())
}
