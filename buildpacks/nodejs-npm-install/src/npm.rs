use commons::fun_run::CmdError;
use libcnb::Env;
use std::path::PathBuf;
use std::process::Command;

pub(crate) struct SetCacheConfig<'a> {
    pub(crate) env: &'a Env,
    pub(crate) cache_dir: PathBuf,
}

impl<'a> From<SetCacheConfig<'a>> for Command {
    fn from(value: SetCacheConfig<'a>) -> Self {
        let mut cmd = Command::new("npm");
        cmd.args([
            "config",
            "set",
            "cache",
            &value.cache_dir.to_string_lossy(),
            "--global",
        ]);
        cmd.envs(value.env);
        cmd
    }
}

#[derive(Debug)]
pub(crate) enum VersionError {
    Command(CmdError),
    Parse(String),
}
pub(crate) struct Version<'a> {
    pub(crate) env: &'a Env,
}

impl<'a> From<Version<'a>> for Command {
    fn from(value: Version<'a>) -> Self {
        let mut cmd = Command::new("npm");
        cmd.arg("--version");
        cmd.envs(value.env);
        cmd
    }
}

pub(crate) struct Install<'a> {
    pub(crate) env: &'a Env,
    pub(crate) with_lockfile: bool,
}

impl<'a> From<Install<'a>> for Command {
    fn from(value: Install<'a>) -> Self {
        let mut cmd = Command::new("npm");
        if value.with_lockfile {
            cmd.arg("ci");
        } else {
            cmd.args(["install", "--no-package-lock"]);
        }
        cmd.arg("--production=false");
        cmd.envs(value.env);
        cmd
    }
}

pub(crate) struct RunScript<'a> {
    pub(crate) env: &'a Env,
    pub(crate) script: String,
}

impl<'a> From<RunScript<'a>> for Command {
    fn from(value: RunScript<'a>) -> Self {
        let mut cmd = Command::new("npm");
        cmd.args(["run", "-s", &value.script]);
        cmd.envs(value.env);
        cmd
    }
}
