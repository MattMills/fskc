use std::time::{Duration, SystemTime};
use sha2::{Sha256, Digest};
use crate::Result;
use super::{DerivedKey, KeyGenerator, SharedContext};

/// Status of key exchange process
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExchangeStatus {
    /// Initial state
    NotStarted,
    /// Context established, generating keys
    GeneratingKeys,
    /// Keys generated, confirming match
    ConfirmingKeys,
    /// Keys confirmed and synchronized
    Complete,
    /// Exchange failed
    Failed,
}

/// Configuration for key exchange
#[derive(Debug, Clone)]
pub struct ExchangeConfig {
    /// Maximum time to wait for confirmation
    pub confirmation_timeout: Duration,
    /// Number of confirmation rounds
    pub confirmation_rounds: usize,
    /// Minimum required key quality
    pub min_key_quality: f64,
}

impl Default for ExchangeConfig {
    fn default() -> Self {
        Self {
            confirmation_timeout: Duration::from_secs(30),
            confirmation_rounds: 3,
            min_key_quality: 0.8,
        }
    }
}

/// Manages key exchange between devices
pub struct KeyExchange {
    config: ExchangeConfig,
    generator: KeyGenerator,
    status: ExchangeStatus,
    derived_key: Option<DerivedKey>,
    confirmation_hashes: Vec<Vec<u8>>,
}

impl KeyExchange {
    /// Create new key exchange with configuration
    pub fn new(config: ExchangeConfig, generator: KeyGenerator) -> Self {
        Self {
            config,
            generator,
            status: ExchangeStatus::NotStarted,
            derived_key: None,
            confirmation_hashes: Vec::new(),
        }
    }

    /// Start key exchange process
    pub fn start_exchange(&mut self, context: &SharedContext) -> Result<()> {
        // Validate context quality
        if context.quality < self.config.min_key_quality {
            self.status = ExchangeStatus::Failed;
            return Err(crate::FskcError::Custom(
                "Insufficient context quality for key exchange".into()
            ));
        }

        // Generate key
        self.status = ExchangeStatus::GeneratingKeys;
        let key = self.generator.generate_key(context)?;

        // Validate key quality
        if key.quality < self.config.min_key_quality {
            self.status = ExchangeStatus::Failed;
            return Err(crate::FskcError::Custom(
                "Generated key does not meet quality requirements".into()
            ));
        }

        self.derived_key = Some(key);
        self.status = ExchangeStatus::ConfirmingKeys;
        Ok(())
    }

    /// Generate confirmation hash for current round
    pub fn generate_confirmation(&mut self, round: usize) -> Result<Vec<u8>> {
        let key = self.derived_key.as_ref().ok_or_else(|| {
            crate::FskcError::Custom("No key available for confirmation".into())
        })?;

        // Generate round-specific hash
        let mut hasher = Sha256::new();
        hasher.update(&key.key);
        hasher.update(&round.to_le_bytes());
        let hash = hasher.finalize().to_vec();

        // Store hash for verification
        self.confirmation_hashes.push(hash.clone());

        Ok(hash)
    }

    /// Verify confirmation hash from other device
    pub fn verify_confirmation(&mut self, round: usize, other_hash: &[u8]) -> Result<bool> {
        // Get our hash for this round
        let our_hash = self.confirmation_hashes.get(round).ok_or_else(|| {
            crate::FskcError::Custom("No confirmation hash for this round".into())
        })?;

        // Compare hashes
        let matches = our_hash == other_hash;

        // Update status if all rounds complete
        if round == self.config.confirmation_rounds - 1 && matches {
            self.status = ExchangeStatus::Complete;
        } else if !matches {
            self.status = ExchangeStatus::Failed;
        }

        Ok(matches)
    }

    /// Get current exchange status
    pub fn status(&self) -> ExchangeStatus {
        self.status
    }

    /// Get exchanged key if available
    pub fn key(&self) -> Option<&DerivedKey> {
        self.derived_key.as_ref()
    }

    /// Reset exchange state
    pub fn reset(&mut self) {
        self.status = ExchangeStatus::NotStarted;
        self.derived_key = None;
        self.confirmation_hashes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entropy::sensor::EntropyQuality;
    use std::time::Duration;

    #[test]
    fn test_key_exchange() -> Result<()> {
        // Create test context
        let context = SharedContext {
            time_window: Duration::from_secs(3),
            measurements: vec![
                super::super::MeasurementWindow {
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

        // Create key exchanges for both devices
        let config = ExchangeConfig::default();
        let mut exchange1 = KeyExchange::new(
            config.clone(),
            KeyGenerator::new(Default::default()),
        );
        let mut exchange2 = KeyExchange::new(
            config,
            KeyGenerator::new(Default::default()),
        );

        // Start exchanges
        exchange1.start_exchange(&context)?;
        exchange2.start_exchange(&context)?;

        // Verify initial status
        assert_eq!(exchange1.status(), ExchangeStatus::ConfirmingKeys);
        assert_eq!(exchange2.status(), ExchangeStatus::ConfirmingKeys);

        // Perform confirmation rounds
        for round in 0..3 {
            let hash1 = exchange1.generate_confirmation(round)?;
            let hash2 = exchange2.generate_confirmation(round)?;

            assert!(exchange1.verify_confirmation(round, &hash2)?);
            assert!(exchange2.verify_confirmation(round, &hash1)?);
        }

        // Verify final status
        assert_eq!(exchange1.status(), ExchangeStatus::Complete);
        assert_eq!(exchange2.status(), ExchangeStatus::Complete);

        // Verify keys match
        let key1 = exchange1.key().unwrap();
        let key2 = exchange2.key().unwrap();
        assert_eq!(key1.key, key2.key);
        assert_eq!(key1.verification_hash, key2.verification_hash);

        Ok(())
    }

    #[test]
    fn test_failed_exchange() -> Result<()> {
        // Create test context with low quality
        let context = SharedContext {
            time_window: Duration::from_secs(1),
            measurements: vec![
                super::super::MeasurementWindow {
                    start_time: SystemTime::now(),
                    duration: Duration::from_secs(1),
                    measurements: vec![0.1],
                    quality: EntropyQuality {
                        shannon_entropy: 0.3,
                        sample_rate: 100.0,
                        signal_to_noise: 2.0,
                        temporal_consistency: 0.5,
                    },
                }
            ],
            quality: 0.3,
            established_at: SystemTime::now(),
        };

        // Create key exchange
        let mut exchange = KeyExchange::new(
            ExchangeConfig::default(),
            KeyGenerator::new(Default::default()),
        );

        // Attempt exchange with low quality context
        let result = exchange.start_exchange(&context);
        assert!(result.is_err());
        assert_eq!(exchange.status(), ExchangeStatus::Failed);

        Ok(())
    }
}
