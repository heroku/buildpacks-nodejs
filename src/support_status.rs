use std::fmt::{Display, Formatter};
use time::Date;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct NodejsVersionInfo {
    pub(crate) status: NodejsVersionStatus,
    pub(crate) end_of_life_date: Date,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NodejsVersionStatus {
    Current,
    ActiveLts,
    MaintenanceLts,
    Eol,
}

impl Display for NodejsVersionStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodejsVersionStatus::Current => write!(f, "Current"),
            NodejsVersionStatus::ActiveLts | NodejsVersionStatus::MaintenanceLts => {
                write!(f, "LTS")
            }
            NodejsVersionStatus::Eol => write!(f, "EOL"),
        }
    }
}
