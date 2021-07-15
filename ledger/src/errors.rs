use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

pub type Result<T> = std::result::Result<T, LedgerError>;

#[derive(Debug)]
pub enum LedgerError {
    TxInvalidHash,
    TxInvalidSignature,
    TxSignatureError,
    TxNoSignature,
    TxInvalidAmount,
    TxSelfTransaction,
    BlockEmptyMerkleRoot,
    BlockInvalid,
    BlockInvalidHeight,
    ChainInvalidHeights,
    ChainPreviousHashMissing,
    ChainInvalidHashes,
    ChainNoGenesis,
    LedgerBalanceError,
}

impl Display for LedgerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LedgerError::TxInvalidHash => write!(f, "invalid transaction hash"),
            LedgerError::TxInvalidSignature => write!(f, "invalid transaction signature"),
            LedgerError::TxSignatureError => write!(f, "error while signing"),
            LedgerError::TxInvalidAmount => write!(f, "transaction amount must be greater than 0"),
            LedgerError::TxSelfTransaction => write!(f, "cannot make transaction to self address"),
            LedgerError::TxNoSignature => write!(f, "transaction is not signed"),
            LedgerError::BlockEmptyMerkleRoot => write!(f, "merkle root is empty"),
            LedgerError::BlockInvalid => write!(f, "block is invalid"),
            LedgerError::BlockInvalidHeight => write!(f, "block height must be greater than 1"),
            LedgerError::ChainInvalidHeights => write!(f, "height of the block is invalid"),
            LedgerError::ChainPreviousHashMissing => {
                write!(f, "previous hash is missing in the block")
            }
            LedgerError::ChainInvalidHashes => write!(f, "previous hash is incorrect"),
            LedgerError::ChainNoGenesis => {
                write!(
                    f,
                    "chain must have a genesis block before adding another block"
                )
            }
            _ => write!(f, "ledger error happened"),
        }
    }
}

impl Error for LedgerError {}
