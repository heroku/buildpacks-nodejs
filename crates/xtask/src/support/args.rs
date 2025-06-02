use node_semver::Version;

pub(crate) fn version_arg(value: &str) -> Result<Version, String> {
    value
        .parse::<Version>()
        .map_err(|_| format!("Failed to parse version: {value}"))
}
