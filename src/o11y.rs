use const_format::formatcp;

const NAMESPACE: &str = "cnb.nodejs";

const DETECT: &str = formatcp!("{NAMESPACE}.detect");

pub(crate) const DETECT_PROVIDES_NODEJS: &str = formatcp!("{DETECT}.provides_nodejs");

pub(crate) const DETECT_REQUIRES_NODEJS: &str = formatcp!("{DETECT}.requires_nodejs");

pub(crate) const PACKAGE_JSON_CONTENTS: &str = formatcp!("{NAMESPACE}.package_json.contents");

pub(crate) const YARNRC_CONTENTS: &str = formatcp!("{NAMESPACE}.yarn.yarnrc_contents");

const RUNTIME: &str = formatcp!("{NAMESPACE}.runtime");

pub(crate) const RUNTIME_REQUESTED_NAME: &str = formatcp!("{RUNTIME}.requested_name");

pub(crate) const RUNTIME_REQUESTED_VERSION: &str = formatcp!("{RUNTIME}.requested_version");

pub(crate) const RUNTIME_NAME: &str = formatcp!("{RUNTIME}.name");

pub(crate) const RUNTIME_VERSION: &str = formatcp!("{RUNTIME}.version");

pub(crate) const RUNTIME_VERSION_MAJOR: &str = formatcp!("{RUNTIME}.version_major");

pub(crate) const RUNTIME_URL: &str = formatcp!("{RUNTIME}.url");

const PACKAGE_MANAGER: &str = formatcp!("{NAMESPACE}.package_manager");

pub(crate) const PACKAGE_MANAGER_REQUESTED_SOURCE: &str =
    formatcp!("{PACKAGE_MANAGER}.requested_source");

pub(crate) const PACKAGE_MANAGER_REQUESTED_NAME: &str =
    formatcp!("{PACKAGE_MANAGER}.requested_name");
pub(crate) const PACKAGE_MANAGER_REQUESTED_VERSION: &str =
    formatcp!("{PACKAGE_MANAGER}.requested_version");
pub(crate) const PACKAGE_MANAGER_NAME: &str = formatcp!("{PACKAGE_MANAGER}.name");
pub(crate) const PACKAGE_MANAGER_VERSION: &str = formatcp!("{PACKAGE_MANAGER}.version");
pub(crate) const PACKAGE_MANAGER_VERSION_MAJOR: &str = formatcp!("{PACKAGE_MANAGER}.version_major");

const CONFIG: &str = formatcp!("{NAMESPACE}.config");

pub(crate) const CONFIG_PRUNE_DEV_DEPENDENCIES_SOURCE: &str =
    formatcp!("{CONFIG}.prune_dev_dependencies_source");
pub(crate) const CONFIG_PRUNE_DEV_DEPENDENCIES_VALUE: &str =
    formatcp!("{CONFIG}.prune_dev_dependencies_value");
pub(crate) const CONFIG_BUILD_SCRIPT_ENABLED_SOURCE: &str =
    formatcp!("{CONFIG}.build_script_enabled_source");
pub(crate) const CONFIG_BUILD_SCRIPT_ENABLED_VALUE: &str =
    formatcp!("{CONFIG}.build_script_enabled_value");

const BUILD_SCRIPTS: &str = formatcp!("{NAMESPACE}.build_scripts");

pub(crate) const BUILD_SCRIPTS_PREBUILD: &str = formatcp!("{BUILD_SCRIPTS}.prebuild");

pub(crate) const BUILD_SCRIPTS_BUILD: &str = formatcp!("{BUILD_SCRIPTS}.build");

pub(crate) const BUILD_SCRIPTS_POSTBUILD: &str = formatcp!("{BUILD_SCRIPTS}.postbuild");
