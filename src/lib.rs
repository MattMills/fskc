//! FractalStateKeyCrypto (fskc) - A cryptographic library using fractal structures and geometric complexity
//! 
//! This library implements a novel cryptographic system based on fractal structures and
//! geometric complexity rather than traditional computational complexity.

mod fractal;
mod roving;
mod crypto;
mod utils;
mod error;

pub use fractal::FractalNode;
pub use roving::RovingSelector;
pub use error::FskcError;

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
