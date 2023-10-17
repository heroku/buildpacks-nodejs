use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
}

impl PackageManager {
    const VALUES: [Self; 3] = [Self::Npm, Self::Pnpm, Self::Yarn];

    #[must_use]
    pub fn lockfile(&self) -> PathBuf {
        match self {
            PackageManager::Npm => PathBuf::from("package-lock.json"),
            PackageManager::Pnpm => PathBuf::from("pnpm-lock.yaml"),
            PackageManager::Yarn => PathBuf::from("yarn.lock"),
        }
    }

    pub fn iterator() -> impl Iterator<Item = PackageManager> {
        Self::VALUES.iter().copied()
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
