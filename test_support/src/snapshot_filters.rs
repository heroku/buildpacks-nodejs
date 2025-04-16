pub(super) fn create_snapshot_filters() -> Vec<(String, String)> {
    vec![
        // ARCH FILTERS
        (r"linux-amd64", "<arch>"),
        (r"linux-arm64", "<arch>"),
        (r"linux-x64", "<arch>"),
        // TIMER FILTERS
        (
            r"- Done \(finished in .*\)",
            "- Done (finished in <time_elapsed>)",
        ),
        (r"\(\d+m \d+s\)", "(<time_elapsed>)"), // (2m 22s)
        (r"\(\d+\.\d+s\)", "(<time_elapsed>)"), // (10.9s)
        (r"\(< 0.\d+s\)", "(<time_elapsed>)"),  // (< 0.3s)
        (r" \.+ ", " ... "),                    // background activity
        // NPM FILTERS
        (
            r"added \d+ packages in (\d+|\d\.\d+)m?s",
            "added <N> packages in <time_elapsed>",
        ),
        (
            r"added \d+ packages, and audited \d+ packages in (\d+|\d\.\d+)m?s",
            "added <NUMBER> packages, and audited <NUMBER> packages in <time_elapsed>",
        ),
        // npm update notifier message. e.g.;
        //
        //       npm notice
        //       npm notice New major version of npm available! 8.19.4 -> 11.3.0
        //       npm notice Changelog: <https://github.com/npm/cli/releases/tag/v11.3.0>
        //       npm notice Run `npm install -g npm@11.3.0` to update!
        //       npm notice
        //
        (
            r"npm notice\s+npm notice New major version.*\s+(?:npm notice.*\s+)+npm notice[^\n]+",
            "<NPM UPDATE NOTIFIER>",
        ),
        // PNPM FILTERS
        (
            r"Progress: resolved \d+, reused \d+, downloaded \d+, added \d+",
            "Progress: resolved <NUMBER>, reused <NUMBER>, downloaded <NUMBER>, added <NUMBER>",
        ),
        (r"Done in (\d+|\d\.\d+)m?s", "Done in <time_elapsed>"),
        (r"╭─+╮\s+(?:│.*│\s+)+╰─+╯", "<PNPM UPDATE NOTIFIER>"),
        // YARN FILTERS
        (r"Completed in \d+s \d+ms", "Completed in <time_elapsed>"),
        (
            r"Done with warnings in \d+s \d+ms",
            "Done with warnings in <time_elapsed>",
        ),
        (
            r"Done in (\d+s|<time_elapsed>) \d+ms",
            "Done in <time_elapsed>",
        ),
    ]
    .into_iter()
    .map(|(matcher, replacement)| (matcher.to_string(), replacement.to_string()))
    .collect()
}
