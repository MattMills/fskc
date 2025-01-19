mod config;
mod layer;

pub use config::LayerConfig;
pub use layer::{Layer, SymmetricLayer, FractalLayer};

use crate::{Result, EntropyBuilder};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::sync::{Arc, Mutex};

/// Manages layered encryption with configurable sequences and parameters
pub struct LayeredCrypto {
    entropy: Arc<Mutex<crate::entropy::CombinedEntropy>>,
    config: LayerConfig,
}

impl LayeredCrypto {
    /// Creates a new LayeredCrypto instance with default configuration
    pub fn new(master_seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(master_seed);
        let entropy = EntropyBuilder::new()
            .add_rng(rng, "Master RNG")
            .build();

        Self {
            entropy,
            config: LayerConfig::default(),
        }
    }

    /// Creates a new LayeredCrypto instance with custom configuration
    pub fn with_config(master_seed: u64, config: LayerConfig) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(master_seed);
        let entropy = EntropyBuilder::new()
            .add_rng(rng, "Master RNG")
            .build();

        Self {
            entropy,
            config,
        }
    }

    /// Creates a new LayeredCrypto instance with custom entropy sources
    pub fn with_entropy(entropy: Arc<Mutex<crate::entropy::CombinedEntropy>>, config: LayerConfig) -> Self {
        Self {
            entropy,
            config,
        }
    }

    /// Encrypts data using the configured layer sequence
    pub fn encrypt(&mut self, data: &[u8], initial_seed: u64) -> Result<Vec<u8>> {
        let mut current = data.to_vec();

        // Apply each layer in sequence
        for layer in &self.config.sequence {
            current = match layer {
                Layer::Fractal(config) => {
                    let node = config.generate(
                        current,
                        initial_seed,
                        self.config.fractal_depth,
                        self.config.chunk_size,
                    )?;
                    node.decrypt()?
                },
                Layer::Symmetric(algo) => {
                    {
                        let mut entropy = self.entropy.lock().unwrap();
                        algo.encrypt(&mut *entropy, &current)?
                    }
                }
            };
        }

        // Apply self-zippering if configured
        if self.config.use_zippering {
            let zipper_seed = initial_seed ^ 
                u64::from_le_bytes(current[..8].try_into().unwrap_or([0; 8]));
            let final_node = FractalLayer::new().generate(
                current,
                zipper_seed,
                self.config.fractal_depth,
                self.config.chunk_size,
            )?;
            current = final_node.decrypt()?;
        }

        Ok(current)
    }

    /// Decrypts data using the configured layer sequence in reverse
    pub fn decrypt(&mut self, data: &[u8], initial_seed: u64) -> Result<Vec<u8>> {
        let mut current = data.to_vec();

        // Remove self-zippering if configured
        if self.config.use_zippering {
            let zipper_seed = initial_seed ^ 
                u64::from_le_bytes(current[..8].try_into().unwrap_or([0; 8]));
            let node = FractalLayer::new().generate(
                current,
                zipper_seed,
                self.config.fractal_depth,
                self.config.chunk_size,
            )?;
            current = node.decrypt()?;
        }

        // Apply layers in reverse
        for layer in self.config.sequence.iter().rev() {
            current = match layer {
                Layer::Fractal(config) => {
                    let node = config.generate(
                        current,
                        initial_seed,
                        self.config.fractal_depth,
                        self.config.chunk_size,
                    )?;
                    node.decrypt()?
                },
                Layer::Symmetric(algo) => {
                    {
                        let mut entropy = self.entropy.lock().unwrap();
                        algo.decrypt(&mut *entropy, &current)?
                    }
                }
            };
        }

        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_sequence() {
        let data = b"Test data for layered encryption".to_vec();
        let initial_seed = 12345;
        let master_seed = 67890;

        let config = LayerConfig::builder()
            .add_fractal()
            .add_aes()
            .add_chacha()
            .add_fractal()
            .build();

        let mut crypto = LayeredCrypto::with_config(master_seed, config.clone());
        
        let encrypted = crypto.encrypt(&data, initial_seed).unwrap();
        let mut crypto = LayeredCrypto::with_config(master_seed, config.clone());
        let decrypted = crypto.decrypt(&encrypted, initial_seed).unwrap();
        
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_with_zippering() {
        let data = b"Test data with self-zippering".to_vec();
        let initial_seed = 12345;
        let master_seed = 67890;

        let config = LayerConfig::builder()
            .add_fractal()
            .add_aes()
            .enable_zippering()
            .build();

        let mut crypto = LayeredCrypto::with_config(master_seed, config.clone());
        
        let encrypted = crypto.encrypt(&data, initial_seed).unwrap();
        let mut crypto = LayeredCrypto::with_config(master_seed, config.clone());
        let decrypted = crypto.decrypt(&encrypted, initial_seed).unwrap();
        
        assert_eq!(data, decrypted);
    }
}
