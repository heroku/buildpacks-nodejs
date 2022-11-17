use crate::yarn::Yarn;
use heroku_nodejs_utils::vrs::Version;
use libcnb::Env;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("Couldn't start yarn command: {0}")]
    Spawn(std::io::Error),
    #[error("Couldn't finish yarn  command: {0}")]
    Wait(std::io::Error),
    #[error("Yarn command finished with a non-zero exit code: {0}")]
    Exit(std::process::ExitStatus),
    #[error("Yarn output couldn't be parsed: {0}")]
    Parse(String),
}

pub(crate) fn yarn_version(env: &Env) -> Result<Version, Error> {
    let output = Command::new("yarn")
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

pub(crate) fn yarn_get_cache(yarn_line: &Yarn, env: &Env) -> Result<PathBuf, Error> {
    let mut args = vec!["config", "get"];
    if yarn_line == &Yarn::Yarn1 {
        args.push("cache-folder");
    } else {
        args.push("cacheFolder");
    }
    let output = Command::new("yarn")
        .args(args)
        .envs(env)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::Spawn)?
        .wait_with_output()
        .map_err(Error::Wait)?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(stdout.into())
}

pub(crate) fn yarn_set_cache(yarn_line: &Yarn, cache_path: &Path, env: &Env) -> Result<(), Error> {
    let cache_path_string = cache_path.to_string_lossy();
    let mut args = vec!["config", "set"];
    if yarn_line == &Yarn::Yarn1 {
        args.append(&mut vec!["cache-folder", &cache_path_string]);
    } else {
        args.append(&mut vec!["cacheFolder", &cache_path_string]);
    }

    let mut process = Command::new("yarn")
        .args(args)
        .envs(env)
        .spawn()
        .map_err(Error::Spawn)?;
    let status = process.wait().map_err(Error::Wait)?;
    status.success().then_some(()).ok_or(Error::Exit(status))
}

pub(crate) fn yarn_install(
    yarn_line: &Yarn,
    zero_install: bool,
    yarn_env: &Env,
) -> Result<(), Error> {
    let mut args = vec!["install"];
    if yarn_line == &Yarn::Yarn1 {
        args.push("--frozen-lockfile");
    } else {
        args.push("--immutable");
        args.push("--inline-builds");
        if zero_install {
            args.push("--check-cache");
            args.push("--immutable-cache");
        }
    }

    let mut process = Command::new("yarn")
        .args(args)
        .envs(yarn_env)
        .spawn()
        .map_err(Error::Spawn)?;

    let status = process.wait().map_err(Error::Wait)?;

    status.success().then_some(()).ok_or(Error::Exit(status))
}

pub(crate) fn yarn_run(yarn_env: &Env, script: &str) -> Result<(), Error> {
    let status = Command::new("yarn")
        .arg("run")
        .arg(script)
        .envs(yarn_env)
        .spawn()
        .map_err(Error::Spawn)?
        .wait()
        .map_err(Error::Wait)?;

    status.success().then_some(()).ok_or(Error::Exit(status))
}
