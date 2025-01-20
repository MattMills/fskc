//! FractalStateKeyCrypto (fskc) - A cryptographic library using fractal structures and geometric complexity
//! 
//! This library implements a novel cryptographic system based on fractal structures and
//! geometric complexity rather than traditional computational complexity.
#![ allow(warnings)]
mod fractal;
mod roving;
mod crypto;
mod utils;
mod error;
mod layered;
pub mod entropy;
mod holographic;
pub mod inside_out;
pub mod binary_container;
pub mod zkp_container;
pub mod vm;
pub mod microcontroller;
pub mod enclave;

pub use fractal::FractalNode;
pub use roving::RovingSelector;
pub use error::FskcError;
pub use layered::{
    LayeredCrypto,
    LayerConfig,
    Layer,
    SymmetricLayer,
    FractalLayer,
};
pub use entropy::{
    EntropySource,
    EntropyBuilder,
    RngEntropy,
    PhysicalEntropy,
};
pub use holographic::HolographicKeyPackage;
pub use holographic::compute::{HomomorphicCompute, Operation};
pub use inside_out::{ComputePair, SystemState};
pub use binary_container::{BinaryContainer, VerificationResult};
pub use zkp_container::ZkpContainer;
pub use enclave::{BlockContext, ExecutionMode, MemoryRegion, ProtectionLevel};

/// Result type for FSKC operations
pub type Result<T> = std::result::Result<T, FskcError>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        // Basic sanity test
        assert!(true);
    }
}
