use std::time::SystemTime;
use sha2::{Sha256, Digest};
use crate::Result;
use super::{SharedContext, MeasurementWindow};

/// Key material derived from shared context
#[derive(Debug, Clone)]
pub struct DerivedKey {
    /// Raw key bytes
    pub key: Vec<u8>,
    /// Key generation timestamp
    pub generated_at: SystemTime,
    /// Key quality score (0.0 to 1.0)
    pub quality: f64,
    /// Key verification hash
    pub verification_hash: Vec<u8>,
}

/// Configuration for key generation
#[derive(Debug, Clone)]
pub struct KeyGenConfig {
    /// Desired key length in bytes
    pub key_length: usize,
    /// Minimum required key quality
    pub min_quality: f64,
    /// Number of hash iterations for KDF
    pub hash_iterations: usize,
}

impl Default for KeyGenConfig {
    fn default() -> Self {
        Self {
            key_length: 32,  // 256-bit keys
            min_quality: 0.8,
            hash_iterations: 10000,
        }
    }
}

/// Manages key generation from shared context
pub struct KeyGenerator {
    config: KeyGenConfig,
}

impl KeyGenerator {
    /// Create new key generator with configuration
    pub fn new(config: KeyGenConfig) -> Self {
        Self { config }
    }

    /// Generate key from shared context
    pub fn generate_key(&self, context: &SharedContext) -> Result<DerivedKey> {
        // Validate context quality
        if context.quality < self.config.min_quality {
            return Err(crate::FskcError::Custom(
                "Insufficient context quality for key generation".into()
            ));
        }

        // Collect entropy from all measurement windows
        let mut entropy = Vec::new();
        for window in &context.measurements {
            entropy.extend(self.extract_entropy(window));
        }

        // Apply key derivation function
        let key = self.derive_key(&entropy)?;

        // Calculate verification hash
        let mut hasher = Sha256::new();
        hasher.update(&key);
        let verification_hash = hasher.finalize().to_vec();

        Ok(DerivedKey {
            key,
            generated_at: SystemTime::now(),
            quality: context.quality,
            verification_hash,
        })
    }

    /// Extract entropy from measurement window
    fn extract_entropy(&self, window: &MeasurementWindow) -> Vec<u8> {
        let mut entropy = Vec::new();
        
        // Convert measurements to bytes with whitening
        for measurement in &window.measurements {
            // Scale to 0-255 range
            let byte = (measurement * 255.0) as u8;
            
            // Simple whitening: XOR with quality metrics
            let whitened = byte ^ 
                (window.quality.shannon_entropy as u8) ^
                (window.quality.signal_to_noise as u8) ^
                (window.quality.temporal_consistency as u8);
            
            entropy.push(whitened);
        }

        entropy
    }

    /// Apply key derivation function to entropy
    fn derive_key(&self, entropy: &[u8]) -> Result<Vec<u8>> {
        let mut key = entropy.to_vec();

        // Apply multiple rounds of hashing
        for _ in 0..self.config.hash_iterations {
            let mut hasher = Sha256::new();
            hasher.update(&key);
            key = hasher.finalize().to_vec();
        }

        // Truncate or pad to desired length
        if key.len() >= self.config.key_length {
            key.truncate(self.config.key_length);
        } else {
            while key.len() < self.config.key_length {
                key.push(0); // Pad with zeros if needed
            }
        }

        Ok(key)
    }

    /// Verify key matches verification hash
    pub fn verify_key(&self, key: &DerivedKey, verification_hash: &[u8]) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&key.key);
        let hash = hasher.finalize();
        hash.as_slice() == verification_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use crate::entropy::sensor::EntropyQuality;

    #[test]
    fn test_key_generation() -> Result<()> {
        // Create test context
        let context = SharedContext {
            time_window: Duration::from_secs(3),
            measurements: vec![
                MeasurementWindow {
                    start_time: SystemTime::now(),
                    duration: Duration::from_secs(1),
                    measurements: vec![0.1, 0.2, 0.3, 0.4, 0.5],
                    quality: EntropyQuality {
                        shannon_entropy: 0.9,
                        sample_rate: 100.0,
                        signal_to_noise: 10.0,
                        temporal_consistency: 1.0,
                    },
                }
            ],
            quality: 0.95,
            established_at: SystemTime::now(),
        };

        // Generate key
        let generator = KeyGenerator::new(KeyGenConfig::default());
        let key = generator.generate_key(&context)?;

        // Verify key properties
        assert_eq!(key.key.len(), 32);
        assert!(key.quality >= 0.8);
        assert!(!key.verification_hash.is_empty());

        // Verify key verification
        assert!(generator.verify_key(&key, &key.verification_hash));

        Ok(())
    }

    #[test]
    fn test_key_verification() -> Result<()> {
        let config = KeyGenConfig {
            key_length: 16,
            min_quality: 0.7,
            hash_iterations: 1000,
        };
        let generator = KeyGenerator::new(config);

        // Create test context
        let context = SharedContext {
            time_window: Duration::from_secs(1),
            measurements: vec![
                MeasurementWindow {
                    start_time: SystemTime::now(),
                    duration: Duration::from_secs(1),
                    measurements: vec![0.5, 0.6, 0.7],
                    quality: EntropyQuality {
                        shannon_entropy: 0.8,
                        sample_rate: 100.0,
                        signal_to_noise: 8.0,
                        temporal_consistency: 0.9,
                    },
                }
            ],
            quality: 0.85,
            established_at: SystemTime::now(),
        };

        // Generate and verify key
        let key = generator.generate_key(&context)?;
        assert!(generator.verify_key(&key, &key.verification_hash));

        // Modify key and verify failure
        let mut bad_key = key.clone();
        bad_key.key[0] ^= 1;
        assert!(!generator.verify_key(&bad_key, &key.verification_hash));

        Ok(())
    }
}
