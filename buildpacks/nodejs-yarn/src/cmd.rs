use heroku_nodejs_utils::vrs::Version;
use libcnb::Env;
use std::{
    fmt,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum YarnLine {
    Yarn1,
    Yarn2,
    Yarn3,
    Yarn4,
}

impl YarnLine {
    pub(crate) fn new(major_version: u64) -> Result<Self, std::io::Error> {
        match major_version {
            1 => Ok(YarnLine::Yarn1),
            2 => Ok(YarnLine::Yarn2),
            3 => Ok(YarnLine::Yarn3),
            4 => Ok(YarnLine::Yarn4),
            x => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown Yarn major version: {x}"),
            )),
        }
    }
}

impl fmt::Display for YarnLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            YarnLine::Yarn1 => "1",
            YarnLine::Yarn2 => "2",
            YarnLine::Yarn3 => "3",
            YarnLine::Yarn4 => "4",
        };
        write!(f, "yarn {v}.x.x")
    }
}

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
        .arg("version")
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

pub(crate) fn yarn_set_cache(
    yarn_line: &YarnLine,
    cache_path: &Path,
    env: &Env,
) -> Result<(), Error> {
    let cache_path_string = cache_path.to_string_lossy();
    let mut args = vec!["config", "set"];
    if yarn_line == &YarnLine::Yarn1 {
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
    yarn_line: &YarnLine,
    zero_install: bool,
    yarn_env: &Env,
) -> Result<(), Error> {
    let mut args = vec!["install"];
    if yarn_line == &YarnLine::Yarn1 {
        args.append(&mut vec!["--frozen-lockfile"]);
    } else {
        args.append(&mut vec!["--immutable"]);
        if zero_install {
            args.append(&mut vec!["--check-cache", "--immutable-cache"]);
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
