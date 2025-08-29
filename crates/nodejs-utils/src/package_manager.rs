use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
}

impl PackageManager {
    #[must_use]
    pub fn lockfile(&self) -> PathBuf {
        match self {
            PackageManager::Npm => PathBuf::from("package-lock.json"),
            PackageManager::Pnpm => PathBuf::from("pnpm-lock.yaml"),
            PackageManager::Yarn => PathBuf::from("yarn.lock"),
        }
    }
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManager::Npm => write!(f, "npm"),
            PackageManager::Pnpm => write!(f, "pnpm"),
            PackageManager::Yarn => write!(f, "Yarn"),
        }
    }
}
