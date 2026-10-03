use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum LightningToolError {
    #[error("Failed to parse BOLT11 invoice: {0}")]
    InvoiceParseError(String),

    #[error("Invoice signature is invalid or corrupted")]
    InvalidSignature,

    #[error("Invoice expired at timestamp {expiry_timestamp}")]
    InvoiceExpired { expiry_timestamp: u64 },

    #[error("Channel error: {0}")]
    ChannelError(String),

    #[error("Wallet error: {0}")]
    WalletError(String),

    #[error("Network connection error: {0}")]
    NetworkError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
