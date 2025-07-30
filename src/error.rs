//! # Error Handling
//! This module defines the error types used throughout the pump.fun monitor service.


use thiserror::Error;

/// Comprehensive error type for all possible failures in the monitor service.
///

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("RPC client error: {0}")]
    RpcClient(#[from] solana_client::client_error::ClientError),

    #[error("WebSocket connection error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Borsh deserialization error: {0}")]
    Borsh(#[from] std::io::Error),

    #[error("Failed to parse string to Pubkey")]
    PubkeyParse,

    #[error("Transaction parsing failed: {0}")]
    TransactionParse(String),

    #[error("Required data not found in transaction: {0}")]
    DataNotFound(String),
}

/// type alias for Results using error type.
///
pub type Result<T> = std::result::Result<T, MonitorError>;