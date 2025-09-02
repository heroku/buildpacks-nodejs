// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use heroku_nodejs_utils::available_parallelism::available_parallelism_env;
use libcnb::data::exec_d::ExecDProgramOutputKey;
use libcnb::exec_d::write_exec_d_program_output;
use std::collections::HashMap;

fn main() {
    let mut output: HashMap<ExecDProgramOutputKey, String> = HashMap::with_capacity(1);
    let (available_parallelism_env_key, available_parallelism_env_value) =
        available_parallelism_env();
    if let Ok(exec_d_output_key) = available_parallelism_env_key.parse::<ExecDProgramOutputKey>() {
        output.insert(exec_d_output_key, available_parallelism_env_value);
    }
    write_exec_d_program_output(output);
}
