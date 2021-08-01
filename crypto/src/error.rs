use thiserror::Error;

#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("invalid signature")]
    InvalidSignature,
}
