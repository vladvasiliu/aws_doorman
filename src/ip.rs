use log::{info, error};
use std::{net::IpAddr, error::Error as StdError, result::Result as StdResult, fmt};

use external_ip::{get_ip};
use std::fmt::Formatter;

pub async fn guess() -> IPGuessResult {
    match get_ip().await {
        Some(ip) => {
            info!("Got external IP: {}", ip);
            Ok(ip)
        },
        None => {
            error!("Failed to guess external IP.");
            Err(IPGuessError::Failed)
        }
    }
}


#[derive(Debug)]
pub enum IPGuessError {
    Failed
}

impl StdError for IPGuessError {}

impl fmt::Display for IPGuessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to get external IP.")
    }
}

pub type IPGuessResult = StdResult<IpAddr, IPGuessError>;
