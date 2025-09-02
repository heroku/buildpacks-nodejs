use fun_run::CmdError;
use libcnb::Env;
use std::path::PathBuf;
use std::process::Command;

pub(crate) struct SetCacheConfig<'a> {
    pub(crate) env: &'a Env,
    pub(crate) cache_dir: &'a PathBuf,
}

impl SetCacheConfig<'_> {
    pub(crate) fn into_command(self) -> Command {
        self.into()
    }
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
    Parse(String, heroku_nodejs_utils::vrs::VersionError),
}

pub(crate) struct Version<'a> {
    pub(crate) env: &'a Env,
}

impl Version<'_> {
    pub(crate) fn into_command(self) -> Command {
        self.into()
    }
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
}

impl Install<'_> {
    pub(crate) fn into_command(self) -> Command {
        self.into()
    }
}

impl<'a> From<Install<'a>> for Command {
    fn from(value: Install<'a>) -> Self {
        let mut cmd = Command::new("npm");
        cmd.arg("ci");
        cmd.envs(value.env);
        cmd
    }
}

pub(crate) struct RunScript<'a> {
    pub(crate) env: &'a Env,
    pub(crate) script: String,
}

impl RunScript<'_> {
    pub(crate) fn into_command(self) -> Command {
        self.into()
    }
}

impl<'a> From<RunScript<'a>> for Command {
    fn from(value: RunScript<'a>) -> Self {
        let mut cmd = Command::new("npm");
        cmd.args(["run", &value.script]);
        cmd.envs(value.env);
        cmd
    }
}

pub(crate) struct Prune<'a> {
    pub(crate) env: &'a Env,
}

impl Prune<'_> {
    pub(crate) fn into_command(self) -> Command {
        self.into()
    }
}

impl<'a> From<Prune<'a>> for Command {
    fn from(value: Prune<'a>) -> Self {
        let mut cmd = Command::new("npm");
        cmd.arg("prune");
        cmd.env("NODE_ENV", "production");
        cmd.envs(value.env);
        cmd
    }
}
