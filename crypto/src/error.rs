use thiserror::Error;

pub type Result<T> = std::result::Result<T, SignatureError>;

#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("invalid signature")]
    InvalidSignature,
}
