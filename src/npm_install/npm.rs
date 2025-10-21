use libcnb::Env;
use std::process::Command;

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
