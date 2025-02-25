use crate::yarn::Yarn;
use bullet_stream::state::SubBullet;
use bullet_stream::{style, Print};
use fun_run::{CmdError, CommandWithName};
use heroku_nodejs_utils::vrs::Version;
use libcnb::Env;
use std::io::Stdout;
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

/// Execute `yarn --version` to determine what version of `yarn` is in effect
/// for this codebase.
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

/// Execute `yarn config get` to determine where the yarn cache is.
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

/// Execute `yarn config set` to set the yarn cache to a specfic location.
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

/// Execute `yarn config set enableGlobalCache false`. This setting is
/// only available on yarn >= 2. If set to `true`, the `cacheFolder` setting
/// will be ignored, and cached dependencies will be stored in the global
/// Yarn cache (`$HOME/.yarn/berry/cache` by default), which isn't
/// persisted into the build cache or the final image. Yarn 2.x and 3.x have
/// a default value to `false`. Yarn 4.x has a default value of `true`.
pub(crate) fn yarn_disable_global_cache(yarn_line: &Yarn, env: &Env) -> Result<(), Error> {
    if yarn_line == &Yarn::Yarn1 {
        return Ok(());
    }
    let mut process = Command::new("yarn")
        .args(["config", "set", "enableGlobalCache", "false"])
        .envs(env)
        .spawn()
        .map_err(Error::Spawn)?;
    let status = process.wait().map_err(Error::Wait)?;
    status.success().then_some(()).ok_or(Error::Exit(status))
}

/// Execute `yarn install` to install dependencies for a yarn project.
pub(crate) fn yarn_install(
    yarn_line: &Yarn,
    zero_install: bool,
    yarn_env: &Env,
    mut log: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, CmdError> {
    let mut args = vec!["install"];
    if yarn_line == &Yarn::Yarn1 {
        args.push("--production=false");
        args.push("--frozen-lockfile");
    } else {
        args.push("--immutable");
        args.push("--inline-builds");
        if zero_install {
            args.push("--immutable-cache");
        }
    }

    log.stream_cmd(Command::new("yarn").args(args).envs(yarn_env))?;

    Ok(log)
}

/// Execute `yarn run` commands like `build`.
pub(crate) fn yarn_run(
    yarn_env: &Env,
    script: &str,
    mut log: Print<SubBullet<Stdout>>,
) -> Result<Print<SubBullet<Stdout>>, CmdError> {
    log.stream_with(
        format!("Running {script} script", script = style::value(script)),
        |stdout, stderr| {
            Command::new("yarn")
                .arg("run")
                .arg(script)
                .envs(yarn_env)
                .stream_output(stdout, stderr)
        },
    )?;
    Ok(log)
}
