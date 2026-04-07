use crate::o11y::*;
use crate::package_json::PackageJson;
use crate::runtimes::nodejs::{NODEJS_INVENTORY, NodejsArtifact};
use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use crate::{BuildpackBuildContext, BuildpackResult, runtimes};
use bullet_stream::global::print;
use bullet_stream::style;
use chrono::Utc;
use indoc::formatdoc;
use libcnb::Env;
use libherokubuildpack::inventory::artifact::{Arch, Os};
use nodejs_data::{NodeReleaseSchedule, VersionRange};
use std::env::consts;
use std::sync::LazyLock;
use tracing::instrument;

static NODE_RELEASE_SCHEDULE: LazyLock<NodeReleaseSchedule> = LazyLock::new(|| {
    NodeReleaseSchedule::from_json(include_str!("../inventory/schedule.json"))
        .expect("Release schedule file should be valid")
});

static OS: LazyLock<Os> = LazyLock::new(|| consts::OS.parse::<Os>().expect("OS should be valid"));

static ARCH: LazyLock<Arch> =
    LazyLock::new(|| consts::ARCH.parse::<Arch>().expect("ARCH should be valid"));

pub(crate) enum RequestedRuntime {
    NodeJsEngine(VersionRange),
    NodeJsDefault(VersionRange),
}

#[instrument(skip_all)]
pub(crate) fn determine_runtime(package_json: &PackageJson) -> RequestedRuntime {
    if let Some(Ok(version)) = package_json.node_engine() {
        tracing::info!(
            { RUNTIME_REQUESTED_NAME } = "nodejs",
            { RUNTIME_REQUESTED_VERSION } = version.to_string(),
            "runtime"
        );
        RequestedRuntime::NodeJsEngine(version)
    } else {
        let requirement = NODE_RELEASE_SCHEDULE
            .newest_supported_lts(Utc::now())
            .expect("Release schedule should contain at least one supported LTS version")
            .requirement
            .clone();
        tracing::info!(
            { RUNTIME_REQUESTED_NAME } = "nodejs",
            { RUNTIME_REQUESTED_VERSION } = "default",
            "runtime"
        );
        RequestedRuntime::NodeJsDefault(requirement)
    }
}

pub(crate) fn log_requested_runtime(requested_runtime: &RequestedRuntime) {
    match requested_runtime {
        RequestedRuntime::NodeJsEngine(version) => {
            print::sub_bullet(format!(
                "Detected Node.js version range: {}",
                style::value(version.to_string())
            ));
        }
        RequestedRuntime::NodeJsDefault(requirement) => {
            print::sub_bullet(format!(
                "Node.js version not specified, using {}",
                // The requirement parsed from the schedule uses the `v` prefixed version range notation.
                // Rewrite this for the currently expected build log output (e.g. v24 => 24.x)
                style::value(format!(
                    "{}.x",
                    requirement.to_string().trim_start_matches('v')
                ))
            ));
        }
    }
}

pub(crate) enum ResolvedRuntime {
    Nodejs(NodejsArtifact),
}

#[instrument(skip_all)]
pub(crate) fn resolve_runtime(
    requested_runtime: RequestedRuntime,
) -> BuildpackResult<ResolvedRuntime> {
    match requested_runtime {
        RequestedRuntime::NodeJsEngine(requirement)
        | RequestedRuntime::NodeJsDefault(requirement) => resolve_nodejs_runtime(&requirement),
    }
}

fn resolve_nodejs_runtime(requirement: &VersionRange) -> BuildpackResult<ResolvedRuntime> {
    let artifact = NODEJS_INVENTORY
        .resolve(*OS, *ARCH, requirement)
        .ok_or(create_unknown_nodejs_version_error(requirement))?;
    tracing::info!(
        { RUNTIME_NAME } = "nodejs",
        { RUNTIME_VERSION } = artifact.version.to_string(),
        { RUNTIME_VERSION_MAJOR } = artifact.version.major(),
        { RUNTIME_URL } = artifact.url,
        "runtime"
    );
    Ok(ResolvedRuntime::Nodejs(artifact.clone()))
}

pub(crate) fn log_resolved_runtime(resolved_runtime: &ResolvedRuntime) {
    match resolved_runtime {
        ResolvedRuntime::Nodejs(artifact) => {
            print::sub_bullet(format!(
                "Resolved Node.js version: {}",
                style::value(artifact.version.to_string())
            ));
            let now = Utc::now();
            if let Some(release) = NODE_RELEASE_SCHEDULE.resolve(&artifact.version)
                && release.is_eol(now)
            {
                print::warning(formatdoc! {"
                    Node.js {} reached its official End-of-Life (EOL) on {}.
                    It no longer receives security updates, bug fixes, or support from the
                    Node.js project and is no longer supported on Heroku.

                    In a future buildpack release, this warning will become a build error.
                    Please upgrade to a supported version as soon as possible to avoid
                    build failures.

                    Supported LTS releases: {}

                    For more information, see:
                    {}",
                    release.requirement,
                    release.end_of_life.format("%B %e, %Y"),
                    NODE_RELEASE_SCHEDULE.supported_lts_labels(now).join(", "),
                    style::url("https://devcenter.heroku.com/articles/nodejs-support#supported-node-js-versions")
                });
            }
        }
    }
}

fn create_unknown_nodejs_version_error(requirement: &VersionRange) -> ErrorMessage {
    let node_releases_url = style::url(format!(
        "https://github.com/nodejs/node/releases?q=\"v{requirement}\"&expanded=true"
    ));
    let inventory_url =
        style::url("https://github.com/heroku/buildpacks-nodejs/blob/inventory/nodejs.toml");
    let version = style::value(requirement.to_string());
    error_message()
        .id("runtime/unknown_nodejs_version")
        .error_type(UserFacing(SuggestRetryBuild::No, SuggestSubmitIssue::Yes))
        .header(format!("Unknown Node.js version: {version}"))
        .body(formatdoc! {"
            The Node.js version provided could not be resolved to a known release in this buildpack's \
            inventory of Node.js releases.

            Suggestions:
            - Confirm if this is a valid Node.js release at {node_releases_url}
            - Check if this buildpack includes the requested Node.js version in its inventory file at {inventory_url}
        "})
        .create()
}

#[instrument(skip_all)]
pub(crate) fn install_runtime(
    context: &BuildpackBuildContext,
    env: &mut Env,
    resolved_runtime: ResolvedRuntime,
) -> BuildpackResult<()> {
    match resolved_runtime {
        ResolvedRuntime::Nodejs(artifact) => {
            // TODO: confirm installation and version by calling `node --version`
            runtimes::nodejs::install(context, env, &artifact)?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::error_handling::test_util::assert_error_snapshot;

    #[test]
    fn unknown_nodejs_version_error() {
        assert_error_snapshot(&create_unknown_nodejs_version_error(
            &VersionRange::parse("0.0.0").unwrap(),
        ));
    }

    #[test]
    fn node_release_schedule_parses() {
        assert!(
            !NODE_RELEASE_SCHEDULE
                .supported_lts_labels(Utc::now())
                .is_empty()
        );
    }
}
