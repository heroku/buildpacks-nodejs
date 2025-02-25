use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::CommandWithName;
use heroku_nodejs_utils::vrs::{Version, VersionError};
use libcnb::Env;
use std::io::Stdout;
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
    mut log: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, fun_run::CmdError> {
    let shim_path_string = shim_path.to_string_lossy();
    let mut command = Command::new("corepack");
    command.args([
        "enable",
        "--install-directory",
        &shim_path_string,
        package_manager,
    ]);
    command.envs(env);

    log = log.sub_bullet(format!("Executing {}", style::command(command.name())));

    command.named_output()?;

    Ok(log)
}

/// Execute `corepack prepare` to install the correct package manager
pub(crate) fn corepack_prepare(
    env: &Env,
    mut log: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, fun_run::CmdError> {
    let mut command = Command::new("corepack");
    command.arg("prepare");
    command.envs(env);
    log.stream_with(
        format!("Running {}", style::command(command.name())),
        |stdout, stderr| command.stream_output(stdout, stderr),
    )?;
    Ok(log)
}
