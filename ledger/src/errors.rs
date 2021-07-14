use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, LedgerError>;

#[derive(Debug)]
pub enum LedgerError {
    TxInvalidHash,
    TxInvalidSignature,
    TxSignatureError,
    TxSelfTransaction,
    BlockEmptyMerkleRoot,
    BlockInvalid,
}

impl Display for LedgerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LedgerError::TxInvalidHash => write!(f, "invalid transaction hash"),
            LedgerError::TxInvalidSignature => write!(f, "invalid transaction signature"),
            LedgerError::TxSignatureError => write!(f, "error while signing"),
            LedgerError::TxSelfTransaction => write!(f, "cannot make transaction to self address"),
            LedgerError::BlockEmptyMerkleRoot => write!(f, "merkle root is empty"),
            LedgerError::BlockInvalid => write!(f, "block is invalid"),
        }
    }
}

impl Error for LedgerError {}
