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
    filters.push((r"Done in (\d+|\d\.\d+)m?s", "Done in <time_elapsed>"));

    filters
        .into_iter()
        .map(|(matcher, replacement)| (matcher.to_string(), replacement.to_string()))
        .collect()
}
