use commons as _;
use fastrand::alphanumeric;
use fun_run::{CmdError, CommandWithName, NamedOutput};
use heroku_nodejs_utils as _;
use indoc as _;
use libcnb as _;
#[cfg(test)]
use libcnb_test as _;
use serde as _;
#[cfg(test)]
use serde_json as _;
use std::env::temp_dir;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
#[cfg(test)]
use test_support as _;

fn main() {
    if let Err(e) = disable_update_notifier() {
        println!("{e}");
    }

    if let Err(e) = set_temp_cache_dir() {
        println!("{e}");
    }
}

fn disable_update_notifier() -> Result<NamedOutput, CmdError> {
    Command::new("npm")
        .args(["config", "set", "update-notifier", "false"])
        .named_output()
}

fn set_temp_cache_dir() -> Result<NamedOutput, CmdError> {
    Command::new("npm")
        .args(["config", "set", "cache"])
        .arg(create_temp_dir_name("npm.XXXXXXX"))
        .named_output()
}

fn create_temp_dir_name(template: &str) -> PathBuf {
    let mut dir_name = OsString::with_capacity(template.len());
    for c in template.chars() {
        dir_name.push(if c == 'X' {
            alphanumeric().to_string()
        } else {
            c.to_string()
        });
    }
    temp_dir().join(dir_name)
}
