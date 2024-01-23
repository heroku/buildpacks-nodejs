use crate::node;
use commons::output::build_log::StartedLogger;
use commons::output::fmt;
use commons::output::fmt::DEBUG_INFO;
use indoc::formatdoc;
use std::fmt::Display;

pub const USE_DEBUG_INFORMATION_AND_RETRY_BUILD: &str = "\
Use the debug information above to troubleshoot and retry your build.";

pub const SUBMIT_AN_ISSUE: &str = "\
If the issue persists and you think you found a bug in the buildpack then reproduce the issue \
locally with a minimal example and open an issue in the buildpack's GitHub repository with the details.";

pub fn print_error_details(
    logger: Box<dyn StartedLogger>,
    error: &impl Display,
) -> Box<dyn StartedLogger> {
    logger
        .section(DEBUG_INFO)
        .step(&error.to_string())
        .end_section()
}

pub fn on_get_node_version_error(error: node::GetNodeVersionError, logger: Box<dyn StartedLogger>) {
    match error {
        node::GetNodeVersionError::Command(e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Failed to determine {node} version information.
    
                    An unexpected error occurred while executing {node_version}. 

                    {USE_DEBUG_INFORMATION_AND_RETRY_BUILD}

                    {SUBMIT_AN_ISSUE}
                ", node = fmt::value("Node"), node_version = fmt::value(e.name()) });
        }
        node::GetNodeVersionError::Parse(stdout, e) => {
            print_error_details(logger, &e)
                .announce()
                .error(&formatdoc! {"
                    Failed to parse {node} version information.
        
                    An unexpected error occurred while parsing version information from {output}. 
                    
                    {SUBMIT_AN_ISSUE}
                ", node = fmt::value("Node"), output = fmt::value(stdout) });
        }
    }
}
