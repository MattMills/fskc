use thiserror::Error;
use rand::Error as RngError;
use getrandom::Error as GetRandomError;

/// Custom error types for FSKC operations
#[derive(Error, Debug)]
pub enum FskcError {
    #[error("Invalid fractal depth: {0}")]
    InvalidDepth(usize),

    #[error("Invalid data size or chunk size: {0}")]
    InvalidDataSize(usize),

    #[error("Invalid seed value")]
    InvalidSeed,

    #[error("Invalid number of particles: {0}")]
    InvalidParticles(usize),

    #[error("Geometric space error: {0}")]
    GeometricError(String),

    #[error("RNG error: {0}")]
    RngError(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Decryption error: {0}")]
    DecryptionError(String),

    #[error("Roving selector error: {0}")]
    RovingError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<RngError> for FskcError {
    fn from(error: RngError) -> Self {
        FskcError::RngError(error.to_string())
    }
}

impl From<FskcError> for RngError {
    fn from(_: FskcError) -> Self {
        RngError::from(GetRandomError::UNSUPPORTED)
    }
}
