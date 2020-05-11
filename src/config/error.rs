use std::error::Error;
use std::fmt;
use std::net::AddrParseError;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum ConfigError {
    MalformedSG,
    MalformedInstance,
    MalformedProtocol,
    MalformedIP(AddrParseError),
    MalformedPort(ParseIntError),
    IncorrectPortRange(String),
}

impl Error for ConfigError {}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedInstance => write!(f, "Wrong instance id format"),
            Self::MalformedSG => write!(f, "Wrong security group id format"),
            Self::MalformedProtocol => write!(f, "Wrong protocol"),
            Self::MalformedIP(err) => write!(f, "Failed to parse IP address: {}", err),
            Self::MalformedPort(err) => write!(f, "Failed to parse port number: {}", err),
            Self::IncorrectPortRange(err) => write!(f, "Incorrect port range: {}", err),
        }
    }
}

impl From<AddrParseError> for ConfigError {
    fn from(err: AddrParseError) -> Self {
        Self::MalformedIP(err)
    }
}
