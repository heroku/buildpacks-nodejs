use crate::utils::vrs::{Requirement, Version};
use libherokubuildpack::inventory::Inventory;
use libherokubuildpack::inventory::artifact::Artifact;
use sha2::Sha256;
use std::sync::LazyLock;

pub(crate) static NODEJS_INVENTORY: LazyLock<NodejsInventory> = LazyLock::new(|| {
    toml::from_str(include_str!("../../inventory/nodejs.toml"))
        .expect("Inventory file should be valid")
});

// TODO: Requirement should capture the original requirement string for display purposes but it
//       doesn't (yet), so this wrapper type will have to do for now.
pub(crate) static DEFAULT_NODEJS_REQUIREMENT: LazyLock<DefaultNodeRequirement> =
    LazyLock::new(|| {
        let current_lts = "22.x";
        DefaultNodeRequirement {
            value: current_lts.to_string(),
            requirement: Requirement::parse(current_lts)
                .expect("Default Node.js version should be valid"),
        }
    });

pub(crate) struct DefaultNodeRequirement {
    pub(crate) value: String,
    pub(crate) requirement: Requirement,
}

pub(crate) type NodejsArtifact = Artifact<Version, Sha256, Option<()>>;

pub(crate) type NodejsInventory = Inventory<Version, Sha256, Option<()>>;
