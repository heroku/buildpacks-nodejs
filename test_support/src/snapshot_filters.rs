#[allow(clippy::vec_init_then_push)]
pub(super) fn create_snapshot_filters() -> Vec<(String, String)> {
    let mut filters: Vec<(&str, &str)> = vec![];

    // [misc] Filter out architectures from output and download urls. e.g.;
    // - Downloading Node.js `22.14.0 (linux-amd64)`
    // - https://nodejs.org/download/release/v22.14.0/node-v22.14.0-linux-x64.tar.gz
    filters.push((r"linux-(?:amd64|arm64|x64)", "<arch>"));

    // [bullet-stream] Timer from streamed command output completion. e.g.;
    // - Done (finished in 3m 29s)
    filters.push((
        r"- Done \(finished in .*\)",
        "- Done (finished in <time_elapsed>)",
    ));

    // [bullet-stream] Timers from background progress completion. e.g.;
    // - (2m 22s)
    // - (10.9s)
    // - (< 0.3s)
    filters.push((
        r"(?:\(\d+m \d+s\)|\(\d+\.\d+s\)|\(< 0.\d+s\))",
        "(<time_elapsed>)",
    ));

    // [bullet-stream] Dots from background activity
    filters.push((r" \.+ ", " ... "));

    // [Yarn] Post `yarn install` timer output. e.g.;
    // - Done in 30s 9ms
    //
    // NOTE: This can conflict with the pnpm version of this filter, so ensure it's listed before
    filters.push((r"Done in \d+s \d+ms", "Done in <time_elapsed>"));

    // [Yarn] Post `yarn install` timer output when warnings are present. e.g.;
    // - Done with warnings in 30s 9ms
    filters.push((
        r"Done with warnings in \d+s \d+ms",
        "Done with warnings in <time_elapsed>",
    ));

    // [Yarn] Step completion with timer shown during `yarn install`. e.g.;
    // - Done with warnings in 30s 9ms
    //
    // NOTE: Sometimes Yarn just emits "Completed" with no timing so that simpler form
    //       is preferred as the replacement value here.
    filters.push((r"Completed in \d+s \d+ms", "Completed"));

    // [npm] Summary of added packages with no audit information. e.g.;
    // - added 12 packages in 27s
    // - added 3 packages in 1.13ms
    filters.push((
        r"added \d+ packages in (\d+|\d\.\d+)m?s",
        "added <NUMBER> packages in <time_elapsed>",
    ));

    // [npm] Summary of added packages with audit information. e.g.;
    // - added 12 packages, and audited 7 packages in 27s
    // - added 3 packages, and audited 11 packages in 1.13ms
    filters.push((
        r"added \d+ packages, and audited \d+ packages in (\d+|\d\.\d+)m?s",
        "added <NUMBER> packages, and audited <NUMBER> packages in <time_elapsed>",
    ));

    // [pnpm] Final progress messages for installed packages. e.g.;
    // - Progress: resolved 9, reused 2, downloaded 7, added 4, done
    //
    // NOTE: This progress message is emitted non-deterministically, so better to remove it entirely
    //       from the snapshot output. It provides little value over other captured pnpm output.
    filters.push((r" +Progress: resolved .*\n", ""));

    // [pnpm] Message shown when lockfile is up to date sometimes has a linebreak, and sometimes it doesn't. e.g.;
    // -      Lockfile is up to date, resolution step is skipped
    //              Packages: +2
    //        ++
    // -      Lockfile is up to date, resolution step is skipped      Packages: +2
    //        ++
    filters.push((
        r"( +)Lockfile is up to date, resolution step is skipped[^\n]+Packages",
        "${1}Lockfile is up to date, resolution step is skipped\n${1}${1}Packages",
    ));

    // [pnpm] Post `pnpm install` timer output. e.g.;
    // - Done in 27s
    // - Done in 3.7ms
    filters.push((r"Done in (?:\d+|\d+\.\d+)m?s", "Done in <time_elapsed>"));

    // [pnpm] Native module compilation output from node-gyp. Similar to the npm version above except
    //        pnpm adds a bit more decoration to each line about the specific module being rebuilt. e.g.;
    // -     .../node_modules/dtrace-provider install$ node-gyp rebuild || node suppress-error.js
    //       .../node_modules/dtrace-provider install: gyp info it worked if it ends with ok
    //       .../node_modules/dtrace-provider install: gyp info using node-gyp@10.1.0
    //       .../node_modules/dtrace-provider install: gyp info using node@20.19.0 | linux | x64
    //       .../node_modules/dtrace-provider install: gyp info find Python using Python version 3.12.3 found at "/usr/bin/python3"
    //       .../node_modules/dtrace-provider install: gyp http GET https://nodejs.org/download/release/v20.19.0/node-v20.19.0-headers.tar.gz
    //       .../node_modules/dtrace-provider install: gyp http 200 https://nodejs.org/download/release/v20.19.0/node-v20.19.0-headers.tar.gz
    //       .../node_modules/dtrace-provider install: gyp http GET https://nodejs.org/download/release/v20.19.0/SHASUMS256.txt
    //       .../node_modules/dtrace-provider install: gyp http 200 https://nodejs.org/download/release/v20.19.0/SHASUMS256.txt
    //       .../node_modules/dtrace-provider install: gyp info spawn /usr/bin/python3
    //       .../node_modules/dtrace-provider install: gyp info spawn args [
    //       .../node_modules/dtrace-provider install: gyp info spawn args '/layers/heroku_nodejs-corepack/mgr/cache/v1/pnpm/9.1.1/dist/node_modules/node-gyp/gyp/gyp_main.py',
    //       .../node_modules/dtrace-provider install: gyp info spawn args 'binding.gyp',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-f',
    //       .../node_modules/dtrace-provider install: gyp info spawn args 'make',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-I',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '/layers/heroku_nodejs-pnpm-install/virtual/store/dtrace-provider@0.8.8/node_modules/dtrace-provider/build/config.gypi',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-I',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '/layers/heroku_nodejs-corepack/mgr/cache/v1/pnpm/9.1.1/dist/node_modules/node-gyp/addon.gypi',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-I',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '/home/heroku/.cache/node-gyp/20.19.0/include/node/common.gypi',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Dlibrary=shared_library',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Dvisibility=default',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Dnode_root_dir=/home/heroku/.cache/node-gyp/20.19.0',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Dnode_gyp_dir=/layers/heroku_nodejs-corepack/mgr/cache/v1/pnpm/9.1.1/dist/node_modules/node-gyp',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Dnode_lib_file=/home/heroku/.cache/node-gyp/20.19.0/<(target_arch)/node.lib',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Dmodule_root_dir=/layers/heroku_nodejs-pnpm-install/virtual/store/dtrace-provider@0.8.8/node_modules/dtrace-provider',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Dnode_engine=v8',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '--depth=.',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '--no-parallel',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '--generator-output',
    //       .../node_modules/dtrace-provider install: gyp info spawn args 'build',
    //       .../node_modules/dtrace-provider install: gyp info spawn args '-Goutput_dir=.'
    //       .../node_modules/dtrace-provider install: gyp info spawn args ]
    //       .../node_modules/dtrace-provider install: gyp info spawn make
    //       .../node_modules/dtrace-provider install: gyp info spawn args [ 'BUILDTYPE=Release', '-C', 'build' ]
    //       .../node_modules/dtrace-provider install: make: Entering directory '/layers/heroku_nodejs-pnpm-install/virtual/store/dtrace-provider@0.8.8/node_modules/dtrace-provider/build'
    //       .../node_modules/dtrace-provider install:   TOUCH Release/obj.target/DTraceProviderStub.stamp
    //       .../node_modules/dtrace-provider install: make: Leaving directory '/layers/heroku_nodejs-pnpm-install/virtual/store/dtrace-provider@0.8.8/node_modules/dtrace-provider/build'
    //       .../node_modules/dtrace-provider install: gyp info ok
    //       .../node_modules/dtrace-provider install: Done
    //
    // NOTE: This pattern must use the non-greedy form of `*?` to capture lines between the start of
    //       node-gyp output and "gyp info ok". The greedy form (`+` or `*`) can end up consuming
    //       more than expected if the output contains more than one node-gyp output section (e.g.;
    //       during a rebuild).
    filters.push((
        r"( *).*gyp info.*\n(?:.*\n)*? *.*gyp info ok",
        "${1}<NODE-GYP BUILD OUTPUT>",
    ));

    filters
        .into_iter()
        .map(|(matcher, replacement)| (matcher.to_string(), replacement.to_string()))
        .collect()
}
