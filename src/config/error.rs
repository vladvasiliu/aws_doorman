use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    IncorrectPortRange(String),
}

impl Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IncorrectPortRange(err) => write!(f, "Incorrect port range: {}", err),
        }
    }
}
