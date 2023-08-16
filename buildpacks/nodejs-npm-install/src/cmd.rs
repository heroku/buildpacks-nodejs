use heroku_nodejs_utils::vrs::Version;
use libcnb::Env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, ExitStatus, Output, Stdio};

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) enum Error {
    Exit(ExitStatus),
    Parse(String),
    Spawn(std::io::Error),
    Wait(std::io::Error),
}

pub(crate) fn npm_version(env: &Env) -> Result<Version> {
    npm_with_output(["--version"], env).and_then(|output| {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .parse::<Version>()
            .map_err(|_| Error::Parse(stdout.into_owned()))
    })
}

pub(crate) fn npm_set_cache_config(env: &Env, location: &Path) -> Result<()> {
    npm(
        [
            "config",
            "set",
            "cache",
            &location.to_string_lossy(),
            "--global",
        ],
        env,
    )
}

pub(crate) fn npm_set_no_audit(env: &Env) -> Result<()> {
    npm(["config", "set", "audit", "false", "--global"], env)
}

pub(crate) fn npm_install(env: &Env) -> Result<()> {
    npm(["ci", "--production=false"], env)
}

pub(crate) fn npm_run(env: &Env, script: &String) -> Result<()> {
    npm(["run", script], env)
}

fn npm<I, S>(args: I, env: &Env) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new("npm")
        .args(args)
        .envs(env)
        .spawn()
        .map_err(Error::Spawn)
        .and_then(|mut child| child.wait().map_err(Error::Wait))
        .and_then(|status| status.success().then_some(()).ok_or(Error::Exit(status)))
}

fn npm_with_output<I, S>(args: I, env: &Env) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new("npm")
        .args(args)
        .envs(env)
        .stdout(Stdio::piped())
        .spawn()
        .map_err(Error::Spawn)
        .and_then(|child| child.wait_with_output().map_err(Error::Wait))
        .and_then(|output| {
            let status = output.status;
            status
                .success()
                .then_some(output)
                .ok_or(Error::Exit(status))
        })
}
