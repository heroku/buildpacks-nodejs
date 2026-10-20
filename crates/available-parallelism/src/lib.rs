use libcnb::data::exec_d::ExecDProgramOutputKey;
use std::ffi::OsString;
use std::fs;
use std::num::NonZeroUsize;

const HEROKU_AVAILABLE_PARALLELISM: &str = "HEROKU_AVAILABLE_PARALLELISM";

#[must_use]
pub fn env_name() -> OsString {
    HEROKU_AVAILABLE_PARALLELISM.into()
}

#[must_use]
pub fn program_output_key() -> ExecDProgramOutputKey {
    HEROKU_AVAILABLE_PARALLELISM
        .parse::<ExecDProgramOutputKey>()
        .expect("HEROKU_AVAILABLE_PARALLELISM should be a valid ExecDProgramOutputKey")
}

#[must_use]
pub fn env_value() -> String {
    value().to_string()
}

fn value() -> usize {
    let fallback = std::thread::available_parallelism().map_or(1, NonZeroUsize::get);
    parallelism(|path| fs::read_to_string(path).ok(), fallback)
}

// Rust floors a fractional cgroup v2 `cpu.max` quota (e.g., Fir dynos reserve a small amount
// of quota for the system, so 1 vCPU is reported as 0.9), so read it directly and round up.
fn parallelism(read_file: impl Fn(&str) -> Option<String>, fallback: usize) -> usize {
    read_file("/sys/fs/cgroup/cpu.max")
        .and_then(|contents| {
            let (quota, period) = contents.trim().split_once(' ')?;
            let (quota, period) = (quota.parse::<usize>().ok()?, period.parse::<usize>().ok()?);
            (period > 0).then(|| quota.div_ceil(period))
        })
        .map_or(fallback, |cpus| cpus.clamp(1, fallback))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const CPU_MAX: &str = "/sys/fs/cgroup/cpu.max";

    fn read_from(files: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let files: HashMap<String, String> = files
            .iter()
            .map(|(path, contents)| ((*path).to_string(), (*contents).to_string()))
            .collect();
        move |path| files.get(path).cloned()
    }

    fn parallelism_for(cpu_max: &str, fallback: usize) -> usize {
        parallelism(read_from(&[(CPU_MAX, cpu_max)]), fallback)
    }

    #[test]
    fn test_parallelism() {
        // unlimited: use the fallback
        assert_eq!(parallelism_for("max 100000\n", 8), 8);
        // fractional quotas round up
        assert_eq!(parallelism_for("90000 100000\n", 8), 1);
        assert_eq!(parallelism_for("180000 100000\n", 8), 2);
        assert_eq!(parallelism_for("800000 100000\n", 8), 8);
        assert_eq!(parallelism_for("1440000 100000\n", 16), 15);
        // no trailing newline
        assert_eq!(parallelism_for("180000 100000", 8), 2);
        // quota larger than the fallback is clamped
        assert_eq!(parallelism_for("800000 100000\n", 4), 4);
        // unreadable or invalid: use the fallback
        assert_eq!(parallelism(read_from(&[]), 8), 8);
        assert_eq!(parallelism_for("garbage\n", 8), 8);
        assert_eq!(parallelism_for("90000\n", 8), 8);
        assert_eq!(parallelism_for("90000 0\n", 8), 8);
    }
}
