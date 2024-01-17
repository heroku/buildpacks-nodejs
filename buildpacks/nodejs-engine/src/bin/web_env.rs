// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb::data::exec_d::ExecDProgramOutputKey;
use libcnb::data::exec_d_program_output_key;
use libcnb::exec_d::write_exec_d_program_output;
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::fs;

fn main() {
    write_exec_d_program_output(web_env(read_env("WEB_CONCURRENCY"), read_env("WEB_MEMORY")));
}

fn web_env(
    concurrency: Option<usize>,
    memory: Option<usize>,
) -> HashMap<ExecDProgramOutputKey, String> {
    let available_memory = detect_available_memory();
    let web_memory = memory.unwrap_or_else(|| default_web_memory(available_memory));
    let web_concurrency =
        concurrency.unwrap_or_else(|| calculate_web_concurrency(available_memory, web_memory));

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
    .map_or(512, |max_bytes: usize| {
        cmp::min(129024, max_bytes / 1_048_576)
    })
}

fn default_web_memory(available_memory: usize) -> usize {
    if available_memory > 16384 {
        return 2048;
    }
    512
}

fn calculate_web_concurrency(available_memory: usize, web_memory: usize) -> usize {
    cmp::max(1, available_memory / web_memory)
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
        assert!((1..=63).contains(&web_concurrency));
        assert!([512, 2048].contains(&web_memory));
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

    #[test]
    fn test_default_web_memory() {
        // heroku standard-1x
        assert_eq!(default_web_memory(512), 512);
        // heroku performance-m
        assert_eq!(default_web_memory(2560), 512);
        // heroku performance-l
        assert_eq!(default_web_memory(14336), 512);
        // memory heavy instance
        assert_eq!(default_web_memory(30720), 2048);
        // extra large memory heavy instance
        assert_eq!(default_web_memory(129025), 2048);
    }

    #[test]
    fn test_calculate_web_concurrency() {
        // heroku standard-1x
        assert_eq!(calculate_web_concurrency(512, 512), 1);
        // heroku performance-m
        assert_eq!(calculate_web_concurrency(2560, 512), 5);
        // heroku performance-l
        assert_eq!(calculate_web_concurrency(14336, 512), 28);
        // large memory heavy instance
        assert_eq!(calculate_web_concurrency(63488, 2048), 31);
        // assert that the calculation won't select a value < 1
        assert_eq!(calculate_web_concurrency(512, 2048), 1);
    }
}
