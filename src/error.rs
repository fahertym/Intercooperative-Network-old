// src/error.rs

use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    BlockchainError(String),
    ConsensusError(String),
    GovernanceError(String),
    ShardingError(String),
    NetworkError(String),
    SmartContractError(String),
    VmError(String),
    IoError(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BlockchainError(msg) => write!(f, "Blockchain error: {}", msg),
            Error::ConsensusError(msg) => write!(f, "Consensus error: {}", msg),
            Error::GovernanceError(msg) => write!(f, "Governance error: {}", msg),
            Error::ShardingError(msg) => write!(f, "Sharding error: {}", msg),
            Error::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Error::SmartContractError(msg) => write!(f, "Smart contract error: {}", msg),
            Error::VmError(msg) => write!(f, "VM error: {}", msg),
            Error::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;