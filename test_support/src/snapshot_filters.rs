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

    // [Yarn] Fetching modules not already in the cache. e.g.;
    // -       ➤ YN0000: ┌ Fetch step
    //         ➤ YN0013: │ side-channel@npm:1.0.4 can't be found in the cache and will be fetched from the remote registry
    //         ➤ YN0013: │ send@npm:0.18.0 can't be found in the cache and will be fetched from the remote registry
    //         ➤ YN0000: └ Completed
    filters.push((
        r"( *)➤ YN0013: │ .*\n(?:.*\n)*? *➤ YN0000: └ Completed",
        "${1}➤ YN0013: │ <LIST OF DOWNLOADED MODULES>\n${1}➤ YN0000: └ Completed",
    ));

    // [Yarn] Native module compilation output from node-gyp which is slightly different than the npm
    //        and pnpm versions as it contains log prefixes and other information to identify the package
    //        being compiled. e.g.;
    // -     ➤ YN0000: ┌ Link step
    //       ➤ YN0000: │ ESM support for PnP uses the experimental loader API and is therefore experimental
    //       ➤ YN0007: │ dtrace-provider@npm:0.8.8 must be built because it never has been before or the last one failed
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info it worked if it ends with ok
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info using node-gyp@10.1.0
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info using node@22.14.0 | linux | x64
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info find Python using Python version 3.12.3 found at "/usr/bin/python3"
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDOUT
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp http GET https://nodejs.org/download/release/v22.14.0/node-v22.14.0-headers.tar.gz
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp http 200 https://nodejs.org/download/release/v22.14.0/node-v22.14.0-headers.tar.gz
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp http GET https://nodejs.org/download/release/v22.14.0/SHASUMS256.txt
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp http 200 https://nodejs.org/download/release/v22.14.0/SHASUMS256.txt
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn /usr/bin/python3
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args [
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '/workspace/.yarn/unplugged/node-gyp-npm-10.1.0-bdea7d2ece/node_modules/node-gyp/gyp/gyp_main.py',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args 'binding.gyp',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-f',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args 'make',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-I',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '/workspace/.yarn/unplugged/dtrace-provider-npm-0.8.8-c06c6b4a53/node_modules/dtrace-provider/build/config.gypi',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-I',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '/workspace/.yarn/unplugged/node-gyp-npm-10.1.0-bdea7d2ece/node_modules/node-gyp/addon.gypi',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-I',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '/home/heroku/.cache/node-gyp/22.14.0/include/node/common.gypi',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Dlibrary=shared_library',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Dvisibility=default',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Dnode_root_dir=/home/heroku/.cache/node-gyp/22.14.0',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Dnode_gyp_dir=/workspace/.yarn/unplugged/node-gyp-npm-10.1.0-bdea7d2ece/node_modules/node-gyp',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Dnode_lib_file=/home/heroku/.cache/node-gyp/22.14.0/<(target_arch)/node.lib',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Dmodule_root_dir=/workspace/.yarn/unplugged/dtrace-provider-npm-0.8.8-c06c6b4a53/node_modules/dtrace-provider',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Dnode_engine=v8',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '--depth=.',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '--no-parallel',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '--generator-output',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args 'build',
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args '-Goutput_dir=.'
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args ]
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn make
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info spawn args [ 'BUILDTYPE=Release', '-C', 'build' ]
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDOUT make: Entering directory '/workspace/.yarn/unplugged/dtrace-provider-npm-0.8.8-c06c6b4a53/node_modules/dtrace-provider/build'
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDOUT   TOUCH Release/obj.target/DTraceProviderStub.stamp
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDOUT make: Leaving directory '/workspace/.yarn/unplugged/dtrace-provider-npm-0.8.8-c06c6b4a53/node_modules/dtrace-provider/build'
    //       ➤ YN0000: │ dtrace-provider@npm:0.8.8 STDERR gyp info ok
    //       ➤ YN0000: └ Completed
    //
    // NOTE: This pattern must use the non-greedy form of `*?` to capture lines between the start of
    //       node-gyp output and "gyp info ok". The greedy form (`+` or `*`) can end up consuming
    //       more than expected if the output contains more than one node-gyp output section (e.g.;
    //       during a rebuild).
    filters.push((
        r"( *)➤ YN0000:.*STDERR gyp info.*\n(?:.*\n)*? *➤ YN0000:.*STDERR gyp info ok",
        "${1}➤ YN0000: │ <NODE-GYP BUILD OUTPUT>",
    ));

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

    // [npm] Native module compilation output from node-gyp. e.g.;
    // -     gyp info it worked if it ends with ok
    //       gyp info using node-gyp@10.1.0
    //       gyp info using node@20.19.0 | linux | x64
    //       gyp info find Python using Python version 3.12.3 found at "/usr/bin/python3"
    //
    //       gyp http GET https://nodejs.org/download/release/v20.19.0/node-v20.19.0-headers.tar.gz
    //       gyp http 200 https://nodejs.org/download/release/v20.19.0/node-v20.19.0-headers.tar.gz
    //       gyp http GET https://nodejs.org/download/release/v20.19.0/SHASUMS256.txt
    //       gyp http 200 https://nodejs.org/download/release/v20.19.0/SHASUMS256.txt
    //       gyp info spawn /usr/bin/python3
    //       gyp info spawn args [
    //       gyp info spawn args '/layers/heroku_nodejs-engine/dist/lib/node_modules/npm/node_modules/node-gyp/gyp/gyp_main.py',
    //       gyp info spawn args 'binding.gyp',
    //       gyp info spawn args '-f',
    //       gyp info spawn args 'make',
    //       gyp info spawn args '-I',
    //       gyp info spawn args '/workspace/node_modules/dtrace-provider/build/config.gypi',
    //       gyp info spawn args '-I',
    //       gyp info spawn args '/layers/heroku_nodejs-engine/dist/lib/node_modules/npm/node_modules/node-gyp/addon.gypi',
    //       gyp info spawn args '-I',
    //       gyp info spawn args '/home/heroku/.cache/node-gyp/20.19.0/include/node/common.gypi',
    //       gyp info spawn args '-Dlibrary=shared_library',
    //       gyp info spawn args '-Dvisibility=default',
    //       gyp info spawn args '-Dnode_root_dir=/home/heroku/.cache/node-gyp/20.19.0',
    //       gyp info spawn args '-Dnode_gyp_dir=/layers/heroku_nodejs-engine/dist/lib/node_modules/npm/node_modules/node-gyp',
    //       gyp info spawn args '-Dnode_lib_file=/home/heroku/.cache/node-gyp/20.19.0/<(target_arch)/node.lib',
    //       gyp info spawn args '-Dmodule_root_dir=/workspace/node_modules/dtrace-provider',
    //       gyp info spawn args '-Dnode_engine=v8',
    //       gyp info spawn args '--depth=.',
    //       gyp info spawn args '--no-parallel',
    //       gyp info spawn args '--generator-output',
    //       gyp info spawn args 'build',
    //       gyp info spawn args '-Goutput_dir=.'
    //       gyp info spawn args ]
    //       gyp info spawn make
    //       gyp info spawn args [ 'BUILDTYPE=Release', '-C', 'build' ]
    //       make: Entering directory '/workspace/node_modules/dtrace-provider/build'
    //         TOUCH Release/obj.target/DTraceProviderStub.stamp
    //       make: Leaving directory '/workspace/node_modules/dtrace-provider/build'
    //       gyp info ok
    //
    // NOTE: This pattern must use the non-greedy form of `*?` to capture lines between the start of
    //       node-gyp output and "gyp info ok". The greedy form (`+` or `*`) can end up consuming
    //       more than expected if the output contains more than one node-gyp output section (e.g.;
    //       during a rebuild).
    filters.push((
        r"( *)gyp info.*\n(?:.*\n)*? *gyp info ok",
        "${1}<NODE-GYP BUILD OUTPUT>",
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
    filters.push((r"Done in (\d+|\d\.\d+)m?s", "Done in <time_elapsed>"));

    filters
        .into_iter()
        .map(|(matcher, replacement)| (matcher.to_string(), replacement.to_string()))
        .collect()
}
