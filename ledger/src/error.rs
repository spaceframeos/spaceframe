use crate::account::Address;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LedgerError {
    #[error("height of the block is invalid")]
    ChainInvalidHeights,

    #[error("previous hash is missing in the block")]
    ChainPreviousHashMissing,

    #[error("previous hash is incorrect")]
    ChainInvalidHashes,

    #[error("chain must have a genesis block before adding another block")]
    ChainNoGenesis,

    #[error("error occured while calculating balance of {0:?}")]
    LedgerBalanceError(Option<Address>),

    #[error("Balance check returns errors: {0}")]
    BalanceCheckWithErrors(String),
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("invalid transaction hash")]
    TxInvalidHash,

    #[error("invalid transaction signature")]
    TxInvalidSignature,

    #[error("error while signing the transaction")]
    TxSignatureError,

    #[error("transaction is not signed")]
    TxNoSignature,

    #[error("genesis transactions must not be signed")]
    GenesisSigned,

    #[error("transaction amount must be greater than 0")]
    TxInvalidAmount,

    #[error("cannot make transaction to self address")]
    TxSelfTransaction,
}

#[derive(Error, Debug)]
pub enum BlockError {
    #[error("merkle root is empty")]
    BlockEmptyMerkleRoot,

    #[error("block is invalid")]
    BlockInvalid,

    #[error("block height must be greater than 1")]
    BlockInvalidHeight,

    #[error("No valid proof found")]
    NoProofFound,
}
