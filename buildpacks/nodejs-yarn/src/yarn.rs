use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
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

impl fmt::Display for Yarn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            Yarn::Yarn1 => "1",
            Yarn::Yarn2 => "2",
            Yarn::Yarn3 => "3",
            Yarn::Yarn4 => "4",
        };
        write!(f, "yarn {v}.x.x")
    }
}
