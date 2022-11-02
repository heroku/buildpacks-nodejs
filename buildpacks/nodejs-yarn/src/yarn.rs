use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum Yarn {
    Yarn1,
    Yarn2,
    Yarn3,
    Yarn4,
}

impl Yarn {
    pub(crate) fn new(major_version: u64) -> Result<Self, std::io::Error> {
        match major_version {
            1 => Ok(Yarn::Yarn1),
            2 => Ok(Yarn::Yarn2),
            3 => Ok(Yarn::Yarn3),
            4 => Ok(Yarn::Yarn4),
            x => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown Yarn major version: {x}"),
            )),
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
