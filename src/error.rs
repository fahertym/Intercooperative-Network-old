use thiserror::Error;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Invalid block: {0}")]
    InvalidBlock(String),

    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("Consensus error: {0}")]
    ConsensusError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Smart contract error: {0}")]
    SmartContractError(String),

    #[error("Unauthorized operation: {0}")]
    UnauthorizedError(String),

    #[error("Resource not found: {0}")]
    NotFoundError(String),

    #[error("Mutex lock error")]
    MutexLockError,

    #[error("Empty blockchain")]
    EmptyBlockchain,

    #[error("Sharding error: {0}")]
    ShardingError(String),

    #[error("General error: {0}")]
    GeneralError(String),
}

pub type BlockchainResult<T> = Result<T, BlockchainError>;