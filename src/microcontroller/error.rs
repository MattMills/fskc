use std::fmt;
use crate::FskcError;

/// Memory access error types
#[derive(Debug)]
pub enum MemoryError {
    /// Address out of bounds
    OutOfBounds { addr: usize, size: usize },
    /// Invalid data size
    InvalidSize { expected: usize, got: usize },
    /// Invalid instruction
    InvalidInstruction,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::OutOfBounds { addr, size } => {
                write!(f, "Memory address {} out of bounds (size: {})", addr, size)
            }
            MemoryError::InvalidSize { expected, got } => {
                write!(f, "Invalid data size: expected {}, got {}", expected, got)
            }
            MemoryError::InvalidInstruction => write!(f, "Invalid instruction encoding"),
        }
    }
}

impl std::error::Error for MemoryError {}

impl From<MemoryError> for FskcError {
    fn from(err: MemoryError) -> Self {
        FskcError::Custom(err.to_string())
    }
}

/// CPU execution error types
#[derive(Debug)]
pub enum CpuError {
    /// Invalid instruction
    InvalidInstruction,
    /// Compute engine error
    ComputeError(String),
}

impl fmt::Display for CpuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuError::InvalidInstruction => write!(f, "Invalid instruction"),
            CpuError::ComputeError(msg) => write!(f, "Compute error: {}", msg),
        }
    }
}

impl std::error::Error for CpuError {}

impl From<CpuError> for FskcError {
    fn from(err: CpuError) -> Self {
        FskcError::Custom(err.to_string())
    }
}
