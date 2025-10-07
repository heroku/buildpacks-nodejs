// Required due to: https://github.com/rust-lang/rust/issues/95513
#![allow(unused_crate_dependencies)]

use libcnb::exec_d::write_exec_d_program_output;
use std::collections::HashMap;

fn main() {
    write_exec_d_program_output(HashMap::from([(
        available_parallelism::program_output_key(),
        available_parallelism::env_value(),
    )]));
}
