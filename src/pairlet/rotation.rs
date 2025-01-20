use std::time::{Duration, SystemTime};
use std::collections::VecDeque;
use crate::Result;
use super::{DerivedKey, KeyGenerator, SharedContext, KeyExchange};

/// Status of a rotated key
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyStatus {
    /// Key is currently active
    Active,
    /// Key is expired but retained for history
    Expired,
    /// Key was compromised or invalidated
    Invalidated,
}

/// Historical key record
#[derive(Debug, Clone)]
pub struct KeyRecord {
    /// The key material
    pub key: DerivedKey,
    /// Key status
    pub status: KeyStatus,
    /// When the key was activated
    pub activated_at: SystemTime,
    /// When the key was deactivated (if applicable)
    pub deactivated_at: Option<SystemTime>,
    /// Reason for deactivation (if applicable)
    pub deactivation_reason: Option<String>,
}

/// Configuration for key rotation
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// Maximum key lifetime
    pub max_key_lifetime: Duration,
    /// Context quality threshold for rotation
    pub rotation_quality_threshold: f64,
    /// Maximum number of historical keys to retain
    pub max_history_size: usize,
    /// Minimum time between rotations
    pub min_rotation_interval: Duration,
}

impl Default for RotationConfig {
    fn default() -> Self {
        Self {
            max_key_lifetime: Duration::from_secs(3600), // 1 hour
            rotation_quality_threshold: 0.9,
            max_history_size: 10,
            min_rotation_interval: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Manages key rotation and history
pub struct KeyRotation {
    config: RotationConfig,
    generator: KeyGenerator,
    active_key: Option<KeyRecord>,
    key_history: VecDeque<KeyRecord>,
    last_rotation: Option<SystemTime>,
}

impl KeyRotation {
    /// Create new key rotation manager
    pub fn new(config: RotationConfig, generator: KeyGenerator) -> Self {
        Self {
            config,
            generator,
            active_key: None,
            key_history: VecDeque::new(),
            last_rotation: None,
        }
    }

    /// Check if key rotation is needed
    pub fn needs_rotation(&self, context: &SharedContext) -> bool {
        // Check if we have an active key
        let Some(active) = &self.active_key else {
            return true; // No active key, rotation needed
        };

        // Check key lifetime
        if active.key.generated_at.elapsed().unwrap_or(Duration::ZERO) > self.config.max_key_lifetime {
            return true;
        }

        // Check context quality
        if context.quality >= self.config.rotation_quality_threshold {
            // Check minimum rotation interval
            if let Some(last) = self.last_rotation {
                if last.elapsed().unwrap_or(Duration::ZERO) >= self.config.min_rotation_interval {
                    return true;
                }
            }
        }

        false
    }

    /// Rotate to new key
    pub fn rotate_key(&mut self, context: &SharedContext, exchange: &mut KeyExchange) -> Result<()> {
        // Start key exchange
        exchange.start_exchange(context)?;

        // Perform confirmation rounds
        for round in 0..3 {
            // Generate confirmation hashes
            let hash = exchange.generate_confirmation(round)?;
            
            // Verify confirmation hash
            if !exchange.verify_confirmation(round, &hash)? {
                return Err(crate::FskcError::Custom(
                    "Key confirmation failed".into()
                ));
            }
        }

        // Check exchange status and get key
        if exchange.status() != super::ExchangeStatus::Complete {
            return Err(crate::FskcError::Custom("Key exchange failed".into()));
        }

        let new_key = exchange.key().ok_or_else(|| {
            crate::FskcError::Custom("No key available after exchange".into())
        })?;

        // Create record for new key
        let new_record = KeyRecord {
            key: new_key.clone(),
            status: KeyStatus::Active,
            activated_at: SystemTime::now(),
            deactivated_at: None,
            deactivation_reason: None,
        };

        // Move current key to history if it exists
        if let Some(mut current) = self.active_key.take() {
            current.status = KeyStatus::Expired;
            current.deactivated_at = Some(SystemTime::now());
            current.deactivation_reason = Some("Routine rotation".into());
            
            self.key_history.push_front(current);

            // Trim history if needed
            while self.key_history.len() > self.config.max_history_size {
                self.key_history.pop_back();
            }
        }

        // Set new active key
        self.active_key = Some(new_record);
        self.last_rotation = Some(SystemTime::now());

        Ok(())
    }

    /// Invalidate current key
    pub fn invalidate_key(&mut self, reason: &str) {
        if let Some(mut key) = self.active_key.take() {
            key.status = KeyStatus::Invalidated;
            key.deactivated_at = Some(SystemTime::now());
            key.deactivation_reason = Some(reason.to_string());
            
            self.key_history.push_front(key);

            // Trim history if needed
            while self.key_history.len() > self.config.max_history_size {
                self.key_history.pop_back();
            }
        }
    }

    /// Get active key if available
    pub fn active_key(&self) -> Option<&KeyRecord> {
        self.active_key.as_ref()
    }

    /// Get key history
    pub fn key_history(&self) -> &VecDeque<KeyRecord> {
        &self.key_history
    }

    /// Clear key history
    pub fn clear_history(&mut self) {
        self.key_history.clear();
    }

    /// Recover key from recovery process
    pub fn recover_key(&mut self, record: KeyRecord) -> Result<()> {
        // Move current key to history if it exists
        if let Some(mut current) = self.active_key.take() {
            current.status = KeyStatus::Expired;
            current.deactivated_at = Some(SystemTime::now());
            current.deactivation_reason = Some("Key recovery".into());
            
            self.key_history.push_front(current);

            // Trim history if needed
            while self.key_history.len() > self.config.max_history_size {
                self.key_history.pop_back();
            }
        }

        // Set recovered key as active
        self.active_key = Some(record);
        self.last_rotation = Some(SystemTime::now());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entropy::sensor::EntropyQuality;
    use std::time::Duration;

    fn create_test_context(quality: f64) -> SharedContext {
        SharedContext {
            time_window: Duration::from_secs(3),
            measurements: vec![
                super::super::MeasurementWindow {
                    start_time: SystemTime::now(),
                    duration: Duration::from_secs(1),
                    measurements: vec![0.1, 0.2, 0.3],
                    quality: EntropyQuality {
                        shannon_entropy: quality,
                        sample_rate: 100.0,
                        signal_to_noise: 10.0,
                        temporal_consistency: 1.0,
                    },
                }
            ],
            quality,
            established_at: SystemTime::now(),
        }
    }

    #[test]
    fn test_key_rotation() -> Result<()> {
        let config = RotationConfig {
            max_key_lifetime: Duration::from_secs(1),
            rotation_quality_threshold: 0.8,
            max_history_size: 2,
            min_rotation_interval: Duration::from_secs(0),
        };

        let mut rotation = KeyRotation::new(
            config,
            KeyGenerator::new(Default::default()),
        );

        let context = create_test_context(0.9);
        let mut exchange = KeyExchange::new(
            Default::default(),
            KeyGenerator::new(Default::default()),
        );

        // Initial rotation
        assert!(rotation.needs_rotation(&context));
        rotation.rotate_key(&context, &mut exchange)?;
        assert!(rotation.active_key().is_some());
        assert_eq!(rotation.key_history().len(), 0);

        // Wait for key expiration
        std::thread::sleep(Duration::from_secs(2));

        // Second rotation
        assert!(rotation.needs_rotation(&context));
        rotation.rotate_key(&context, &mut exchange)?;
        assert!(rotation.active_key().is_some());
        assert_eq!(rotation.key_history().len(), 1);
        assert_eq!(rotation.key_history()[0].status, KeyStatus::Expired);

        Ok(())
    }

    #[test]
    fn test_key_invalidation() -> Result<()> {
        let mut rotation = KeyRotation::new(
            RotationConfig::default(),
            KeyGenerator::new(Default::default()),
        );

        let context = create_test_context(0.9);
        let mut exchange = KeyExchange::new(
            Default::default(),
            KeyGenerator::new(Default::default()),
        );

        // Generate initial key
        rotation.rotate_key(&context, &mut exchange)?;
        assert!(rotation.active_key().is_some());

        // Invalidate key
        rotation.invalidate_key("Security breach");
        assert!(rotation.active_key().is_none());
        assert_eq!(rotation.key_history().len(), 1);
        assert_eq!(rotation.key_history()[0].status, KeyStatus::Invalidated);
        assert_eq!(
            rotation.key_history()[0].deactivation_reason,
            Some("Security breach".into())
        );

        Ok(())
    }
}
