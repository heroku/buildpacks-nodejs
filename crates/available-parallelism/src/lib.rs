use libcnb::data::exec_d::ExecDProgramOutputKey;
use std::ffi::OsString;

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
    std::thread::available_parallelism()
        // XXX: The Rust implementation always rounds down the value reported here if the
        //      (quota / period) calculated from cgroups cpu.max produces a fractional value.
        //      For Heroku Fir Dynos this will always end up reducing the cpu allocation
        //      value by 1 since a small amount of quota is reserved for the system so we need
        //      to add that back unless Rust changes how they deal with rounding.
        .map(|value| value.get() + 1)
        .unwrap_or_default()
}
