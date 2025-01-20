use std::time::{Duration, SystemTime};
use crate::Result;
use super::{
    DerivedKey, KeyGenerator, SharedContext, KeyExchange,
    KeyStatus, KeyRecord, rotation::KeyRotation,
};

/// Configuration for key recovery
#[derive(Debug, Clone, Copy)]
pub struct RecoveryConfig {
    /// Maximum age of recoverable context
    pub max_context_age: Duration,
    /// Minimum required context quality for recovery
    pub min_recovery_quality: f64,
    /// Number of confirmation rounds required
    pub confirmation_rounds: usize,
    /// Maximum recovery attempts before lockout
    pub max_recovery_attempts: usize,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            max_context_age: Duration::from_secs(3600 * 24), // 24 hours
            min_recovery_quality: 0.9,
            confirmation_rounds: 5, // More rounds for recovery
            max_recovery_attempts: 3,
        }
    }
}

/// Status of a recovery attempt
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecoveryStatus {
    /// Initial state
    NotStarted,
    /// Verifying historical context
    VerifyingContext,
    /// Generating recovery keys
    GeneratingKeys,
    /// Confirming recovery keys
    ConfirmingKeys,
    /// Recovery complete
    Complete,
    /// Recovery failed
    Failed,
}

/// Manages key recovery operations
pub struct KeyRecovery {
    config: RecoveryConfig,
    generator: KeyGenerator,
    status: RecoveryStatus,
    attempts: usize,
    recovery_started: Option<SystemTime>,
    backup_key: Option<DerivedKey>,
}

impl KeyRecovery {
    /// Create new key recovery manager
    pub fn new(config: RecoveryConfig, generator: KeyGenerator) -> Self {
        Self {
            config,
            generator,
            status: RecoveryStatus::NotStarted,
            attempts: 0,
            recovery_started: None,
            backup_key: None,
        }
    }

    /// Start recovery process
    pub fn start_recovery(&mut self, context: &SharedContext) -> Result<()> {
        // Check if locked out
        if self.attempts >= self.config.max_recovery_attempts {
            return Err(crate::FskcError::Custom(
                "Maximum recovery attempts exceeded".into()
            ));
        }

        // Validate context age
        if context.established_at.elapsed()? > self.config.max_context_age {
            self.status = RecoveryStatus::Failed;
            self.attempts += 1;
            return Err(crate::FskcError::Custom(
                "Context too old for recovery".into()
            ));
        }

        // Validate context quality
        if context.quality < self.config.min_recovery_quality {
            self.status = RecoveryStatus::Failed;
            self.attempts += 1;
            return Err(crate::FskcError::Custom(
                "Insufficient context quality for recovery".into()
            ));
        }

        self.status = RecoveryStatus::VerifyingContext;
        self.recovery_started = Some(SystemTime::now());
        Ok(())
    }

    /// Generate recovery key
    pub fn generate_recovery_key(&mut self, context: &SharedContext) -> Result<DerivedKey> {
        if self.status != RecoveryStatus::VerifyingContext {
            return Err(crate::FskcError::Custom(
                "Recovery not properly initialized".into()
            ));
        }

        self.status = RecoveryStatus::GeneratingKeys;
        let key = self.generator.generate_key(context)?;

        // Store backup key
        self.backup_key = Some(key.clone());

        self.status = RecoveryStatus::ConfirmingKeys;
        Ok(key)
    }

    /// Verify recovery with another device
    pub fn verify_recovery(
        &mut self,
        exchange: &mut KeyExchange,
        rotation: &mut KeyRotation,
    ) -> Result<()> {
        if self.status != RecoveryStatus::ConfirmingKeys {
            return Err(crate::FskcError::Custom(
                "Recovery not ready for verification".into()
            ));
        }

        // Ensure exchange is in correct state
        if exchange.status() != super::ExchangeStatus::ConfirmingKeys {
            return Err(crate::FskcError::Custom(
                "Key exchange not properly initialized".into()
            ));
        }

        // Generate and verify confirmation hash
        let hash = exchange.generate_confirmation(0)?;
        if !exchange.verify_confirmation(0, &hash)? {
            self.status = RecoveryStatus::Failed;
            self.attempts += 1;
            return Err(crate::FskcError::Custom(
                "Recovery verification failed".into()
            ));
        }

        // Get verified key
        if let Some(key) = exchange.key() {
            // Create recovery record
            let record = KeyRecord {
                key: key.clone(),
                status: KeyStatus::Active,
                activated_at: SystemTime::now(),
                deactivated_at: None,
                deactivation_reason: None,
            };

            // Update rotation manager
            rotation.recover_key(record)?;

            self.status = RecoveryStatus::Complete;
            Ok(())
        } else {
            self.status = RecoveryStatus::Failed;
            self.attempts += 1;
            Err(crate::FskcError::Custom("No key available after verification".into()))
        }
    }

    /// Get current recovery status
    pub fn status(&self) -> RecoveryStatus {
        self.status
    }

    /// Get number of recovery attempts
    pub fn attempts(&self) -> usize {
        self.attempts
    }

    /// Check if recovery is locked out
    pub fn is_locked_out(&self) -> bool {
        self.attempts >= self.config.max_recovery_attempts
    }

    /// Get backup key if available
    pub fn backup_key(&self) -> Option<&DerivedKey> {
        self.backup_key.as_ref()
    }

    /// Reset recovery state
    pub fn reset(&mut self) {
        self.status = RecoveryStatus::NotStarted;
        self.recovery_started = None;
        self.backup_key = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entropy::sensor::EntropyQuality;

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
    fn test_key_recovery() -> Result<()> {
        let mut recovery = KeyRecovery::new(
            RecoveryConfig::default(),
            KeyGenerator::new(Default::default()),
        );

        let context = create_test_context(0.95);

        // Start recovery
        recovery.start_recovery(&context)?;
        assert_eq!(recovery.status(), RecoveryStatus::VerifyingContext);

        // Generate recovery key
        let key = recovery.generate_recovery_key(&context)?;
        assert_eq!(recovery.status(), RecoveryStatus::ConfirmingKeys);
        assert!(recovery.backup_key().is_some());

        Ok(())
    }

    #[test]
    fn test_recovery_lockout() -> Result<()> {
        let mut recovery = KeyRecovery::new(
            RecoveryConfig {
                max_recovery_attempts: 2,
                ..Default::default()
            },
            KeyGenerator::new(Default::default()),
        );

        let context = create_test_context(0.7); // Low quality

        // First attempt
        assert!(recovery.start_recovery(&context).is_err());
        assert_eq!(recovery.attempts(), 1);
        assert!(!recovery.is_locked_out());

        // Second attempt
        assert!(recovery.start_recovery(&context).is_err());
        assert_eq!(recovery.attempts(), 2);
        assert!(recovery.is_locked_out());

        // Third attempt should fail
        assert!(recovery.start_recovery(&context).is_err());
        assert_eq!(recovery.status(), RecoveryStatus::Failed);

        Ok(())
    }
}
