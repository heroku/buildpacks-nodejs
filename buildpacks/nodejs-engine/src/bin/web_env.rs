#![warn(clippy::pedantic)]

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

pub fn main() {
    write_exec_d_program_output(web_env());
}

fn web_env() -> HashMap<ExecDProgramOutputKey, String> {
    let web_mem = detect_web_memory();
    let available_mem = detect_available_memory();
    let web_concurrency = calculate_web_concurrency(web_mem, available_mem);

    HashMap::from([
        (
            exec_d_program_output_key!("WEB_CONCURRENCY"),
            web_concurrency.to_string(),
        ),
        (
            exec_d_program_output_key!("WEB_MEMORY"),
            web_mem.to_string(),
        ),
    ])
}

fn calculate_web_concurrency(web_mem: usize, available_mem: usize) -> usize {
    env::var("WEB_CONCURRENCY")
        .ok()
        .and_then(|m| m.parse().ok())
        .unwrap_or_else(|| cmp::max(1, available_mem / web_mem))
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

fn detect_web_memory() -> usize {
    env::var("WEB_MEMORY")
        .ok()
        .and_then(|m| m.parse().ok())
        .unwrap_or(DEFAULT_WEB_MEMORY_MB)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_env_default() {
        env::remove_var("WEB_CONCURRENCY");
        env::remove_var("WEB_MEMORY");
        let web_env = web_env();
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
        env::set_var("WEB_CONCURRENCY", "42");
        env::set_var("WEB_MEMORY", "4242");
        let web_env = web_env();
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
