use std::fmt::Display;

pub(super) fn get_test_arch() -> TestArch {
    std::env::var("INTEGRATION_TEST_CNB_ARCH").map_or_else(
        |_| TestArch::from(std::env::consts::ARCH.to_string()),
        TestArch::from,
    )
}

pub(super) enum TestArch {
    Arm64,
    Amd64,
}

impl Display for TestArch {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TestArch::Arm64 => write!(f, "arm64"),
            TestArch::Amd64 => write!(f, "amd64"),
        }
    }
}

impl From<String> for TestArch {
    fn from(value: String) -> Self {
        match value.as_str() {
            "arm64" | "aarch64" => TestArch::Arm64,
            "amd64" | "x86_64" => TestArch::Amd64,
            _ => unimplemented!("Unsupported test architecture: {value}"),
        }
    }
}
