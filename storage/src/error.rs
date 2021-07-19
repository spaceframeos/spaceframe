use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Path must be a directory")]
    PathIsNotDirectory,

    #[error("Object could not be serialized")]
    SerializationError,

    #[error("File creation failed")]
    FileCreationFailed,

    #[error("Could not write data")]
    DataWriteFailed,
}
