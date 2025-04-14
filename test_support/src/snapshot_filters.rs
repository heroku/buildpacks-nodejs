pub(super) fn create_snapshot_filters(
    snapshot_filters: Vec<(String, String)>,
) -> Vec<(String, String)> {
    let copy_filters = |slice: &[(&str, &str)]| {
        slice
            .iter()
            .map(|(m, r)| ((*m).to_string(), (*r).to_string()))
            .collect::<Vec<_>>()
    };

    let mut filters = vec![];
    filters.extend(copy_filters(&ARCH_FILTERS));
    filters.extend(copy_filters(&TIMER_FILTERS));
    filters.extend(copy_filters(&NPM_FILTERS));
    filters.extend(copy_filters(&PNPM_FILTERS));
    filters.extend(copy_filters(&YARN_FILTERS));
    for (matcher, replacement) in snapshot_filters {
        filters.push((matcher, replacement));
    }
    filters
}

const ARCH_FILTERS: [(&str, &str); 3] = [
    (r"linux-amd64", "<arch>"),
    (r"linux-arm64", "<arch>"),
    (r"linux-x64", "<arch>"),
];

const TIMER_FILTERS: [(&str, &str); 5] = [
    (
        r"- Done \(finished in .*\)",
        "- Done (finished in <time_elapsed>)",
    ),
    (r" \.+ ", " ... "),
    // (2m 22s)
    (r"\(\d+m \d+s\)", "(<time_elapsed>)"),
    // (10.9s)
    (r"\(\d+\.\d+s\)", "(<time_elapsed>)"),
    // (< 0.3s)
    (r"\(< 0.\d+s\)", "(<time_elapsed>)"),
];

const NPM_FILTERS: [(&str, &str); 2] = [
    (
        r"added \d+ packages in (\d+|\d\.\d+)m?s",
        "added <N> packages in <time_elapsed>",
    ),
    (
        r"added \d+ packages, and audited \d+ packages in (\d+|\d\.\d+)m?s",
        "added <N> packages, and audited <M> packages in <time_elapsed>",
    ),
];

const PNPM_FILTERS: [(&str, &str); 2] = [
    (
        r"Progress: resolved \d+, reused \d+, downloaded \d+, added \d+",
        "Progress: resolved <A>, reused <B>, downloaded <C>, added <D>",
    ),
    (r"Done in (\d+|\d\.\d+)m?s", "Done in <time_elapsed>"),
];

const YARN_FILTERS: [(&str, &str); 3] = [
    (r"Completed in \d+s \d+ms", "Completed in <time_elapsed>"),
    (
        r"Done with warnings in \d+s \d+ms",
        "Done with warnings in <time_elapsed>",
    ),
    (
        r"Done in (\d+s|<time_elapsed>) \d+ms",
        "Done in <time_elapsed>",
    ),
];
