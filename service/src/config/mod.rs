use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Production,
    Testing,
    Other(String),
}

impl Environment {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Testing => "testing",
            Self::Other(env) => env,
        }
    }
}

impl From<&str> for Environment {
    fn from(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "development" | "dev" => Self::Development,
            "production" | "prod" => Self::Production,
            "testing" | "test" => Self::Testing,
            _ => Self::Other(s.to_string()),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
