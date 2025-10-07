use crate::runtimes::nodejs::{DEFAULT_NODEJS_REQUIREMENT, NODEJS_INVENTORY, NodejsArtifact};
use crate::utils::error_handling::ErrorType::UserFacing;
use crate::utils::error_handling::{
    ErrorMessage, SuggestRetryBuild, SuggestSubmitIssue, error_message,
};
use crate::utils::vrs::Requirement;
use crate::{BuildpackBuildContext, BuildpackResult, package_json, runtimes};
use bullet_stream::global::print;
use bullet_stream::style;
use indoc::formatdoc;
use libcnb::Env;
use libherokubuildpack::inventory::artifact::{Arch, Os};
use std::env::consts;
use std::path::Path;
use std::sync::LazyLock;

static OS: LazyLock<Os> = LazyLock::new(|| consts::OS.parse::<Os>().expect("OS should be valid"));

static ARCH: LazyLock<Arch> =
    LazyLock::new(|| consts::ARCH.parse::<Arch>().expect("ARCH should be valid"));

pub(crate) enum RequestedRuntime {
    NodeJsEngine(Requirement),
    NodeJsDefault,
}

pub(crate) fn determine_runtime(app_dir: &Path) -> BuildpackResult<RequestedRuntime> {
    let package_json = package_json::PackageJson::try_from(app_dir.join("package.json"))?;
    let requested_node_version = match package_json.node_engine() {
        Some(version) => RequestedRuntime::NodeJsEngine(version),
        None => RequestedRuntime::NodeJsDefault,
    };
    Ok(requested_node_version)
}

pub(crate) fn log_requested_runtime(requested_runtime: &RequestedRuntime) {
    match requested_runtime {
        RequestedRuntime::NodeJsEngine(version) => {
            print::sub_bullet(format!(
                "Detected Node.js version range: {}",
                style::value(version.to_string())
            ));
        }
        RequestedRuntime::NodeJsDefault => {
            print::sub_bullet(format!(
                "Node.js version not specified, using {}",
                style::value(&DEFAULT_NODEJS_REQUIREMENT.value)
            ));
        }
    }
}

pub(crate) enum ResolvedRuntime {
    Nodejs(NodejsArtifact),
}

pub(crate) fn resolve_runtime(
    requested_runtime: RequestedRuntime,
) -> BuildpackResult<ResolvedRuntime> {
    match requested_runtime {
        RequestedRuntime::NodeJsEngine(requirement) => resolve_nodejs_runtime(&requirement),
        RequestedRuntime::NodeJsDefault => {
            resolve_nodejs_runtime(&DEFAULT_NODEJS_REQUIREMENT.requirement)
        }
    }
}

fn resolve_nodejs_runtime(requirement: &Requirement) -> BuildpackResult<ResolvedRuntime> {
    let artifact = NODEJS_INVENTORY
        .resolve(*OS, *ARCH, requirement)
        .ok_or(unknown_nodejs_version_error(requirement))?;
    Ok(ResolvedRuntime::Nodejs(artifact.clone()))
}

pub(crate) fn log_resolved_runtime(resolved_runtime: &ResolvedRuntime) {
    match resolved_runtime {
        ResolvedRuntime::Nodejs(artifact) => print::sub_bullet(format!(
            "Resolved Node.js version: {}",
            style::value(artifact.version.to_string())
        )),
    }
}

fn unknown_nodejs_version_error(requirement: &Requirement) -> ErrorMessage {
    let node_releases_url = style::url(format!(
        "https://github.com/nodejs/node/releases?q=\"v{requirement}\"&expanded=true"
    ));
    let inventory_url =
        style::url("https://github.com/heroku/buildpacks-nodejs/blob/inventory/nodejs.toml");
    let version = style::value(requirement.to_string());
    error_message()
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
    use bullet_stream::strip_ansi;
    use insta::{assert_snapshot, with_settings};
    use test_support::test_name;

    #[test]
    fn test_unknown_nodejs_version_error_message() {
        assert_error_snapshot(&unknown_nodejs_version_error(
            &Requirement::parse("0.0.0").unwrap(),
        ));
    }

    fn assert_error_snapshot(error: &ErrorMessage) {
        let error_message = strip_ansi(error.to_string());
        let test_name = format!("runtime__{}", test_name().split("::").last().unwrap());
        with_settings!({
            prepend_module_to_snapshot => false,
            omit_expression => true,
        }, {
            assert_snapshot!(test_name, error_message);
        });
    }
}
