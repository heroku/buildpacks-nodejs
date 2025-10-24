#![allow(unused_crate_dependencies)]
use libcnb::data::exec_d_program_output_key;
use libcnb::exec_d::write_exec_d_program_output;
use std::collections::HashMap;
use std::env::temp_dir;

fn main() {
    write_exec_d_program_output(HashMap::from([
        // We reconfigure the cache folder which, at build time points to the npm_cache layer,
        // to point to a temp folder at run time since the npm_cache layer is read-only. This
        // is helpful for two reasons:
        // - on startup, npm will try to access the cache folder
        // - npm's logs-dir is configured to also write to a directory named `_logs` inside the cache folder
        //
        // See:
        // - https://docs.npmjs.com/cli/v10/using-npm/config#cache
        // - https://docs.npmjs.com/cli/v10/using-npm/config#logs-dir
        (
            exec_d_program_output_key!("npm_config_cache"),
            temp_dir().join("npm_cache").to_string_lossy().to_string(),
        ),
        // Disable the update notifier at runtime which can (potentially) run at npm startup.
        // See:
        // - https://docs.npmjs.com/cli/v10/using-npm/config#update-notifier
        // - https://github.com/npm/cli/issues/7044,
        // - https://github.com/npm/cli/pull/7061
        (
            exec_d_program_output_key!("npm_config_update-notifier"),
            "false".to_string(),
        ),
    ]));
}
