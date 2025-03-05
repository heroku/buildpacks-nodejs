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
    filters.extend(copy_filters(&PACK_FILTERS));
    filters.extend(copy_filters(&BUILDPACK_FILTERS));
    filters.extend(copy_filters(&TIMER_FILTERS));
    filters.extend(copy_filters(&NPM_FILTERS));
    filters.extend(copy_filters(&PNPM_FILTERS));
    filters.extend(copy_filters(&YARN_FILTERS));
    for (matcher, replacement) in snapshot_filters {
        filters.push((matcher, replacement));
    }
    filters
}

const PACK_FILTERS: [(&str, &str); 4] = [
    (r"libcnbtest_\w+", "<build_image_name>"),
    (r"\*\*\* Images \([\w\d]+\)", "*** Images (<image_hash>)"),
    (
        r"fail: ([\w/-]+)@\d+\.\d+\.\d+ provides unused \w+",
        "fail: ${1}@<version> provides unused <provides_id>",
    ),
    (
        r#"Restoring metadata for "(.*)" from app image"#,
        r#"Restoring metadata for "<buildpack_id>" from app image"#,
    ),
];

const BUILDPACK_FILTERS: [(&str, &str); 10] = [
    (
        r"heroku/nodejs-engine(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-engine${1}<nodejs_engine_version>",
    ),
    (
        r"heroku/nodejs-corepack(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-corepack${1}<nodejs_corepack_version>",
    ),
    (
        r"heroku/nodejs-yarn(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-yarn${1}<nodejs_yarn_version>",
    ),
    (
        r"heroku/nodejs-function-invoker(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-function-invoker${1}<nodejs_function_invoker_version>",
    ),
    (
        r"heroku/nodejs-npm(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-npm${1}<nodejs_npm_version>",
    ),
    (
        r"heroku/nodejs-pnpm-engine(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-pnpm-engine${1}<nodejs_pnpm_engine_version>",
    ),
    (
        r"heroku/nodejs-pnpm-install(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-pnpm-install${1}<nodejs_pnpm_install_version>",
    ),
    (
        r"heroku/nodejs-npm-engine(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-npm-engine${1}<nodejs_npm_engine_version>",
    ),
    (
        r"heroku/nodejs-npm-install(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-npm-install${1}<nodejs_npm_install_version>",
    ),
    (
        r"heroku/nodejs-yarn(\s+)\d+\.\d+\.\d+",
        "heroku/nodejs-yarn${1}<nodejs_yarn_version>",
    ),
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
