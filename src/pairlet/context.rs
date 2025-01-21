use std::time::{Duration, SystemTime};
use crate::Result;
use super::{MeasurementWindow, CoPresenceValidator};

/// Represents shared environmental context between devices
#[derive(Debug, Clone)]
pub struct SharedContext {
    /// Time window of shared context
    pub time_window: Duration,
    /// Environmental measurements
    pub measurements: Vec<MeasurementWindow>,
    /// Context quality score (0.0 to 1.0)
    pub quality: f64,
    /// Timestamp of context establishment
    pub established_at: SystemTime,
}

/// Configuration for context establishment
#[derive(Debug, Clone)]
pub struct ContextConfig {
    /// Minimum required context quality
    pub min_quality: f64,
    /// Number of measurement windows required
    pub required_windows: usize,
    /// Maximum age of measurements
    pub max_age: Duration,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            min_quality: 0.8,
            required_windows: 3,
            max_age: Duration::from_secs(60),
        }
    }
}

/// Manages shared context establishment between devices
pub struct ContextManager {
    config: ContextConfig,
    validator: CoPresenceValidator,
    contexts: Vec<SharedContext>,
}

impl ContextManager {
    /// Create new context manager
    pub fn new(config: ContextConfig, validator: CoPresenceValidator) -> Self {
        Self {
            config,
            validator,
            contexts: Vec::new(),
        }
    }

    /// Attempt to establish shared context with another device
    pub fn establish_context(&mut self, other_windows: &[MeasurementWindow]) -> Result<Option<SharedContext>> {
        // Get our recent measurement windows
        let our_windows = self.validator.recent_windows(self.config.required_windows);
        
        if our_windows.len() < self.config.required_windows {
            return Ok(None); // Not enough measurements yet
        }

        // Calculate average validation metrics across windows
        let mut total_correlation = 0.0;
        let mut total_sync = 0.0;
        let mut total_proximity = 0.0;
        let mut count = 0;

        for (our_window, other_window) in our_windows.iter().zip(other_windows) {
            // Skip old measurements
            if our_window.start_time.elapsed()? > self.config.max_age {
                continue;
            }

            total_correlation += self.validator.calculate_correlation(our_window, other_window);
            total_sync += self.validator.calculate_sync_score(our_window, other_window);
            total_proximity += self.validator.calculate_proximity(our_window, other_window);
            count += 1;
        }

        if count == 0 {
            return Ok(None); // No valid measurement pairs
        }

        // Calculate average quality metrics
        let avg_correlation = total_correlation / count as f64;
        let avg_sync = total_sync / count as f64;
        let avg_proximity = total_proximity / count as f64;

        // Calculate overall context quality
        let quality = (avg_correlation + avg_sync + avg_proximity) / 3.0;

        if quality >= self.config.min_quality {
            // Context establishment successful
            let context = SharedContext {
                time_window: Duration::from_secs(count as u64),
                measurements: our_windows,
                quality,
                established_at: SystemTime::now(),
            };
            self.contexts.push(context.clone());
            Ok(Some(context))
        } else {
            Ok(None)
        }
    }

    /// Get most recently established context
    pub fn current_context(&self) -> Option<&SharedContext> {
        self.contexts.last()
    }

    /// Check if context is still valid
    pub fn is_context_valid(&self, context: &SharedContext) -> bool {
        // Check context age
        if context.established_at.elapsed().unwrap_or(self.config.max_age) > self.config.max_age {
            return false;
        }

        // Check quality threshold
        context.quality >= self.config.min_quality
    }

    /// Get validator reference
    pub fn validator(&self) -> &CoPresenceValidator {
        &self.validator
    }

    /// Get validator mutable reference
    pub fn validator_mut(&mut self) -> &mut CoPresenceValidator {
        &mut self.validator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entropy::sensor::{Accelerometer, Barometer, EntropyQuality};

    #[test]
    fn test_context_establishment() -> Result<()> {
        // Create validators
        let config = ContextConfig::default();
        let mut manager1 = ContextManager::new(
            config.clone(),
            CoPresenceValidator::new(Default::default()),
        );
        let mut manager2 = ContextManager::new(
            config,
            CoPresenceValidator::new(Default::default()),
        );

        // Create test windows with highly correlated data
        let mut windows1 = Vec::new();
        let mut windows2 = Vec::new();
        
        for i in 0..3 {
            let time = SystemTime::now();
            let window1 = MeasurementWindow {
                start_time: time,
                duration: Duration::from_secs(1),
                measurements: vec![1.0 + i as f64 * 0.2, 2.0 + i as f64 * 0.2, 3.0 + i as f64 * 0.2],
                quality: EntropyQuality {
                    shannon_entropy: 1.0,
                    sample_rate: 100.0,
                    signal_to_noise: 10.0,
                    temporal_consistency: 1.0,
                },
            };
            
            let window2 = MeasurementWindow {
                start_time: time,
                duration: Duration::from_secs(1),
                measurements: vec![1.1 + i as f64 * 0.2, 2.1 + i as f64 * 0.2, 3.1 + i as f64 * 0.2],
                quality: EntropyQuality {
                    shannon_entropy: 1.0,
                    sample_rate: 100.0,
                    signal_to_noise: 9.8,
                    temporal_consistency: 1.0,
                },
            };
            
            windows1.push(window1);
            windows2.push(window2);
        }

        // Add simulated windows to validators
        for window in windows1 {
            manager1.validator_mut().add_test_window(window);
        }
        
        for window in windows2.clone() {
            manager2.validator_mut().add_test_window(window);
        }

        // Get windows from second validator for context establishment
        let other_windows = manager2.validator().recent_windows(3);
        
        // Test context establishment with simulated data
        let context = manager1.establish_context(&other_windows)?;
        assert!(context.is_some(), "Context establishment should succeed");

        if let Some(context) = context {
            assert!(context.quality >= 0.8, "Context quality should be high");
            assert!(manager1.is_context_valid(&context), "Context should be valid");
        }

        Ok(())
    }
}
