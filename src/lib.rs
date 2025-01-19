//! FractalStateKeyCrypto (fskc) - A cryptographic library using fractal structures and geometric complexity
//! 
//! This library implements a novel cryptographic system based on fractal structures and
//! geometric complexity rather than traditional computational complexity.

mod fractal;
mod roving;
mod crypto;
mod utils;
mod error;
mod layered;
pub mod entropy;
mod holographic;
pub mod inside_out;

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
