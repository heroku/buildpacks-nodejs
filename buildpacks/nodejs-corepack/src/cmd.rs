use heroku_nodejs_utils::vrs::Version;
use libcnb::Env;
use std::{
    path::Path,
    process::{Command, Stdio},
};

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Couldn't start corepack command: {0}")]
    Spawn(std::io::Error),
    #[error("Couldn't finish corepack  command: {0}")]
    Wait(std::io::Error),
    #[error("Corepack command finished with a non-zero exit code: {0}")]
    Exit(std::process::ExitStatus),
    #[error("Corepack output couldn't be parsed: {0}")]
    Parse(String),
}

/// Execute `corepack --version` to determine corepack version
pub(crate) fn corepack_version(env: &Env) -> Result<Version, Error> {
    let output = Command::new("corepack")
        .arg("--version")
        .envs(env)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::Spawn)?
        .wait_with_output()
        .map_err(Error::Wait)?;

    output
        .status
        .success()
        .then_some(())
        .ok_or(Error::Exit(output.status))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .parse()
        .map_err(|_| Error::Parse(stdout.into_owned()))
}

/// Execute `corepack enable` to setup a corepack shim
pub(crate) fn corepack_enable(
    package_manager: &str,
    shim_path: &Path,
    env: &Env,
) -> Result<(), Error> {
    let shim_path_string = shim_path.to_string_lossy();
    let mut process = Command::new("corepack")
        .args([
            "enable",
            "--install-directory",
            &shim_path_string,
            package_manager,
        ])
        .envs(env)
        .spawn()
        .map_err(Error::Spawn)?;
    let status = process.wait().map_err(Error::Wait)?;
    status.success().then_some(()).ok_or(Error::Exit(status))
}

/// Execute `corepack prepare` to install the correct package manager
pub(crate) fn corepack_prepare(env: &Env) -> Result<(), Error> {
    let mut process = Command::new("corepack")
        .arg("prepare")
        .envs(env)
        .spawn()
        .map_err(Error::Spawn)?;
    let status = process.wait().map_err(Error::Wait)?;
    status.success().then_some(()).ok_or(Error::Exit(status))
}
