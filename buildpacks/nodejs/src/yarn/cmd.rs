use crate::yarn::Yarn;
use bullet_stream::global::print;
use fun_run::{CmdError, CommandWithName};
use heroku_nodejs_utils::vrs::{Version, VersionError};
use libcnb::Env;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug)]
pub(crate) enum YarnVersionError {
    Command(CmdError),
    Parse(String, VersionError),
}

/// Execute `yarn --version` to determine what version of `yarn` is in effect
/// for this codebase.
pub(crate) fn yarn_version(env: &Env) -> Result<Version, YarnVersionError> {
    Command::new("yarn")
        .arg("--version")
        .envs(env)
        .named_output()
        .map_err(YarnVersionError::Command)
        .and_then(|output| {
            let stdout = output.stdout_lossy();
            stdout
                .parse::<Version>()
                .map_err(|e| YarnVersionError::Parse(stdout, e))
        })
}

/// Execute `yarn config get` to determine where the yarn cache is.
pub(crate) fn yarn_get_cache(yarn_line: &Yarn, env: &Env) -> Result<PathBuf, CmdError> {
    let mut cmd = Command::new("yarn");
    cmd.envs(env);

    cmd.arg("config");
    cmd.arg("get");
    if yarn_line == &Yarn::Yarn1 {
        cmd.arg("cache-folder");
    } else {
        cmd.arg("cacheFolder");
    }

    cmd.named_output()
        .map(|output| PathBuf::from(output.stdout_lossy().trim()))
}

/// Execute `yarn config set` to set the yarn cache to a specfic location.
pub(crate) fn yarn_set_cache(
    yarn_line: &Yarn,
    cache_path: &Path,
    env: &Env,
) -> Result<(), CmdError> {
    let cache_path_string = cache_path.to_string_lossy();
    let mut args = vec!["config", "set"];
    if yarn_line == &Yarn::Yarn1 {
        args.append(&mut vec!["cache-folder", &cache_path_string]);
    } else {
        args.append(&mut vec!["cacheFolder", &cache_path_string]);
    }
    print::sub_stream_cmd(Command::new("yarn").args(args).envs(env)).map(|_| ())
}

/// Execute `yarn config set enableGlobalCache false`. This setting is
/// only available on yarn >= 2. If set to `true`, the `cacheFolder` setting
/// will be ignored, and cached dependencies will be stored in the global
/// Yarn cache (`$HOME/.yarn/berry/cache` by default), which isn't
/// persisted into the build cache or the final image. Yarn 2.x and 3.x have
/// a default value to `false`. Yarn 4.x has a default value of `true`.
pub(crate) fn yarn_disable_global_cache(yarn_line: &Yarn, env: &Env) -> Result<(), CmdError> {
    if yarn_line == &Yarn::Yarn1 {
        return Ok(());
    }
    print::sub_stream_cmd(
        Command::new("yarn")
            .args(["config", "set", "enableGlobalCache", "false"])
            .envs(env),
    )
    .map(|_| ())
}

/// Execute `yarn install` to install dependencies for a yarn project.
pub(crate) fn yarn_install(
    yarn_line: &Yarn,
    zero_install: bool,
    yarn_env: &Env,
) -> Result<(), CmdError> {
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

    print::sub_stream_cmd(Command::new("yarn").args(args).envs(yarn_env)).map(|_| ())
}

/// Execute `yarn run` commands like `build`.
pub(crate) fn yarn_run(yarn_env: &Env, script: &str) -> Result<(), CmdError> {
    print::sub_stream_cmd(Command::new("yarn").arg("run").arg(script).envs(yarn_env)).map(|_| ())
}
