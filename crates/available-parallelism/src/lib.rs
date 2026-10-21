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
    cpu_max_parallelism(|path| fs::read_to_string(path).ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, NonZeroUsize::get))
}

// Rust floors a fractional cgroup v2 `cpu.max` quota (e.g., Fir dynos reserve a small amount
// of quota for the system, so 1 vCPU is reported as 0.9), so read it directly and round up.
fn cpu_max_parallelism(read_file: impl Fn(&str) -> Option<String>) -> Option<usize> {
    let contents = read_file("/sys/fs/cgroup/cpu.max")?;
    let (quota, period) = contents.trim().split_once(' ')?;
    let (quota, period) = (quota.parse::<usize>().ok()?, period.parse::<usize>().ok()?);
    (period > 0).then(|| quota.div_ceil(period).max(1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parallelism_for(cpu_max: Option<&str>) -> Option<usize> {
        cpu_max_parallelism(|path| {
            cpu_max
                .filter(|_| path == "/sys/fs/cgroup/cpu.max")
                .map(String::from)
        })
    }

    #[test]
    fn fractional_quota_rounds_up() {
        assert_eq!(parallelism_for(Some("90000 100000\n")), Some(1));
        assert_eq!(parallelism_for(Some("180000 100000\n")), Some(2));
        assert_eq!(parallelism_for(Some("1440000 100000\n")), Some(15));
    }

    #[test]
    fn whole_quota_is_unchanged() {
        assert_eq!(parallelism_for(Some("800000 100000\n")), Some(8));
        assert_eq!(parallelism_for(Some("800000 100000")), Some(8));
    }

    #[test]
    fn tiny_quota_is_at_least_one() {
        assert_eq!(parallelism_for(Some("0 100000\n")), Some(1));
    }

    #[test]
    fn unlimited_quota_has_no_value() {
        assert_eq!(parallelism_for(Some("max 100000\n")), None);
    }

    #[test]
    fn missing_or_invalid_cpu_max_has_no_value() {
        assert_eq!(parallelism_for(None), None);
        assert_eq!(parallelism_for(Some("garbage\n")), None);
        assert_eq!(parallelism_for(Some("90000\n")), None);
        assert_eq!(parallelism_for(Some("90000 0\n")), None);
    }
}
