use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, LedgerError>;

#[derive(Debug)]
pub enum LedgerError {
    TxInvalidHash,
    TxInvalidSignature,
}

impl Display for LedgerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LedgerError::TxInvalidHash => write!(f, "Invalid transaction hash"),
            LedgerError::TxInvalidSignature => write!(f, "Invalid transaction signature"),
        }
    }
}

impl Error for LedgerError {}
