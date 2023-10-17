use commons::fun_run::CmdError;
use libcnb::Env;
use std::process::Command;

#[derive(Debug)]
pub(crate) enum VersionError {
    Command(CmdError),
    Parse(String, heroku_nodejs_utils::vrs::VersionError),
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
