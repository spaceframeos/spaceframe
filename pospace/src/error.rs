use std::io::ErrorKind;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PoSpaceError {
    #[error("Space parameter k must be greater than 12 and less than 48, found {0}")]
    InvalidK(usize),

    #[error("Metadata is empty in plot entry")]
    EmptyMetadata,

    #[error("Position is empty in plot entry")]
    EmptyPosition,

    #[error("Offset is empty in plot entry")]
    EmptyOffset,

    #[error("Too many matches found. Try with another plot seed or a larger k")]
    TooManyMatches,
}

#[derive(Error, Debug)]
pub enum F1CalculatorError {
    #[error("Length of x value must be {expected} bits, found {found} bits")]
    LengthMismatch { expected: usize, found: usize },
}

#[derive(Error, Debug)]
pub enum SortError {
    #[error("Could not rename sorted plot to final plot: {0:?}")]
    RenameError(ErrorKind),

    #[error("Could not delete intermediate file '{0:?}': {1:?}")]
    DeleteError(PathBuf, ErrorKind),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Could not deserialize buffer")]
    DeserializationError,

    #[error("Could not serialize buffer")]
    SerializationError,

    #[error("No more entries")]
    EndOfFile,

    #[error("The content of the plot has been altered")]
    InvalidFileContent,
}

#[derive(Error, Debug)]
pub enum MergeChunkError {
    #[error("Some chunks are empty while trying to chetch the minimum")]
    EmptyChunksWhileFetchingMininum,

    #[error("The minimum chunk is empty")]
    MinChunkIsEmpty,
}
