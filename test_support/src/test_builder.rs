use std::fmt::Display;

pub(super) fn get_test_builder() -> TestBuilder {
    std::env::var("INTEGRATION_TEST_CNB_BUILDER")
        .map(TestBuilder::from)
        .unwrap_or(TestBuilder::Heroku24)
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum TestBuilder {
    Heroku22,
    Heroku24,
    Other(String),
}

impl Display for TestBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TestBuilder::Heroku22 => write!(f, "heroku/builder:22"),
            TestBuilder::Heroku24 => write!(f, "heroku/builder:24"),
            TestBuilder::Other(name) => write!(f, "{name}"),
        }
    }
}

impl From<TestBuilder> for String {
    fn from(value: TestBuilder) -> String {
        value.to_string()
    }
}

impl From<String> for TestBuilder {
    fn from(value: String) -> Self {
        if value == TestBuilder::Heroku22.to_string() {
            TestBuilder::Heroku22
        } else if value == TestBuilder::Heroku24.to_string() {
            TestBuilder::Heroku24
        } else {
            TestBuilder::Other(value)
        }
    }
}
