use libcnb::Env;
use std::process::Command;

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
