// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb::data::exec_d::ExecDProgramOutputKey;
use libcnb::data::exec_d_program_output_key;
use libcnb::exec_d::write_exec_d_program_output;
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::fs;

const MAX_AVAILABLE_MEMORY_MB: usize = 14336;
const DEFAULT_AVAILABLE_MEMORY_MB: usize = 512;
const DEFAULT_WEB_MEMORY_MB: usize = 512;

fn main() {
    write_exec_d_program_output(web_env(read_env("WEB_CONCURRENCY"), read_env("WEB_MEMORY")));
}

fn web_env(
    concurrency: Option<usize>,
    memory: Option<usize>,
) -> HashMap<ExecDProgramOutputKey, String> {
    let available_memory = detect_available_memory();
    let web_memory = memory.unwrap_or(DEFAULT_WEB_MEMORY_MB);
    let web_concurrency = concurrency.unwrap_or_else(|| cmp::max(1, available_memory / web_memory));

    HashMap::from([
        (
            exec_d_program_output_key!("WEB_CONCURRENCY"),
            web_concurrency.to_string(),
        ),
        (
            exec_d_program_output_key!("WEB_MEMORY"),
            web_memory.to_string(),
        ),
    ])
}

fn read_env(key: &str) -> Option<usize> {
    env::var(key).ok().and_then(|var| var.parse().ok())
}

fn detect_available_memory() -> usize {
    [
        "/sys/fs/cgroup/memory.max",
        "/sys/fs/cgroup/memory/memory.limit_in_bytes",
    ]
    .iter()
    .find_map(|path| fs::read_to_string(path).ok())
    .and_then(|contents| contents.trim().parse().ok())
    .map_or(DEFAULT_AVAILABLE_MEMORY_MB, |max_bytes: usize| {
        cmp::min(MAX_AVAILABLE_MEMORY_MB, max_bytes / 1_048_576)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_env_default() {
        let web_env = web_env(None, None);
        let web_concurrency: usize = web_env
            .get("WEB_CONCURRENCY")
            .expect("WEB_CONCURRENCY should exist")
            .parse()
            .expect("WEB_CONCURRENCY should be a number");
        let web_memory: usize = web_env
            .get("WEB_MEMORY")
            .expect("WEB_MEMORY should exist")
            .parse()
            .expect("WEB_MEMORY should be a number");

        println!("WEB_CONCURRENCY: {web_concurrency}");
        assert!((1..=32).contains(&web_concurrency));
        assert!((256..=2048).contains(&web_memory));
    }

    #[test]
    fn test_web_env_does_not_rewrite() {
        let web_env = web_env(Some(42), Some(4242));
        let web_concurrency: usize = web_env
            .get("WEB_CONCURRENCY")
            .expect("WEB_CONCURRENCY should exist")
            .parse()
            .expect("WEB_CONCURRENCY should be a number");
        let web_memory: usize = web_env
            .get("WEB_MEMORY")
            .expect("WEB_MEMORY should exist")
            .parse()
            .expect("WEB_MEMORY should be a number");

        assert_eq!(web_concurrency, 42);
        assert_eq!(web_memory, 4242);
    }
}
