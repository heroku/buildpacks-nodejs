use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum Yarn {
    Yarn1,
    Yarn2,
    Yarn3,
    Yarn4,
}

impl Yarn {
    pub(crate) fn from_major(major_version: u64) -> Option<Self> {
        match major_version {
            1 => Some(Yarn::Yarn1),
            2 => Some(Yarn::Yarn2),
            3 => Some(Yarn::Yarn3),
            4 => Some(Yarn::Yarn4),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) enum NodeLinker {
    Pnp,
    Pnpm,
    NodeModules,
}

impl FromStr for NodeLinker {
    type Err = UnknownNodeLinker;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pnp" => Ok(NodeLinker::Pnp),
            "node-modules" => Ok(NodeLinker::NodeModules),
            "pnpm" => Ok(NodeLinker::Pnpm),
            _ => Err(UnknownNodeLinker(value.to_string())),
        }
    }
}

#[derive(Debug)]
pub(crate) struct UnknownNodeLinker(pub(crate) String);
