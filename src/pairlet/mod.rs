pub mod context;
pub mod keygen;
pub mod exchange;
pub mod rotation;
pub mod recovery;

use std::time::{Duration, SystemTime};
use crate::{Result, entropy::sensor::{Sensor, SensorConfig, EntropyQuality}};
pub use context::{SharedContext, ContextConfig, ContextManager};
pub use keygen::{DerivedKey, KeyGenConfig, KeyGenerator};
pub use exchange::{ExchangeStatus, ExchangeConfig, KeyExchange};
pub use rotation::{KeyStatus, KeyRecord, RotationConfig, KeyRotation};
pub use recovery::{RecoveryStatus, RecoveryConfig, KeyRecovery};

/// Represents a temporal window of sensor measurements
#[derive(Debug, Clone)]
pub struct MeasurementWindow {
    /// Start time of the window
    pub start_time: SystemTime,
    /// Duration of the window
    pub duration: Duration,
    /// Sensor measurements within the window
    pub measurements: Vec<f64>,
    /// Quality metrics for the window
    pub quality: EntropyQuality,
}

/// Configuration for co-presence validation
#[derive(Debug, Clone)]
pub struct CoPresenceConfig {
    /// Minimum correlation threshold
    pub min_correlation: f64,
    /// Maximum time difference allowed
    pub max_time_diff: Duration,
    /// Minimum required proximity
    pub min_proximity: f64,
    /// Window size for measurements
    pub window_size: Duration,
}

impl Default for CoPresenceConfig {
    fn default() -> Self {
        Self {
            min_correlation: 0.8,
            max_time_diff: Duration::from_millis(100),
            min_proximity: 0.9,
            window_size: Duration::from_secs(1),
        }
    }
}

/// Manages physical co-presence validation
pub struct CoPresenceValidator {
    config: CoPresenceConfig,
    sensors: Vec<Box<dyn Sensor>>,
    measurement_windows: Vec<MeasurementWindow>,
}

impl CoPresenceValidator {
    /// Create new validator with configuration
    pub fn new(config: CoPresenceConfig) -> Self {
        Self {
            config,
            sensors: Vec::new(),
            measurement_windows: Vec::new(),
        }
    }

    /// Add a sensor to the validator
    pub fn add_sensor<S: Sensor + 'static>(&mut self, sensor: S) {
        self.sensors.push(Box::new(sensor));
    }

    /// Calculate correlation between two measurement windows
    pub fn calculate_correlation(&self, window1: &MeasurementWindow, window2: &MeasurementWindow) -> f64 {
        if window1.measurements.len() != window2.measurements.len() {
            return 0.0;
        }

        let n = window1.measurements.len() as f64;
        let mean1: f64 = window1.measurements.iter().sum::<f64>() / n;
        let mean2: f64 = window2.measurements.iter().sum::<f64>() / n;

        let mut numerator = 0.0;
        let mut denom1 = 0.0;
        let mut denom2 = 0.0;

        for (x, y) in window1.measurements.iter().zip(window2.measurements.iter()) {
            let dx = x - mean1;
            let dy = y - mean2;
            numerator += dx * dy;
            denom1 += dx * dx;
            denom2 += dy * dy;
        }

        if denom1 == 0.0 || denom2 == 0.0 {
            0.0
        } else {
            numerator / (denom1.sqrt() * denom2.sqrt())
        }
    }

    /// Calculate temporal synchronization score
    pub fn calculate_sync_score(&self, window1: &MeasurementWindow, window2: &MeasurementWindow) -> f64 {
        let time_diff = window1.start_time
            .duration_since(window2.start_time)
            .unwrap_or(Duration::from_secs(0));

        if time_diff > self.config.max_time_diff {
            0.0
        } else {
            1.0 - (time_diff.as_secs_f64() / self.config.max_time_diff.as_secs_f64())
        }
    }

    /// Calculate proximity score based on signal strength and quality
    pub fn calculate_proximity(&self, window1: &MeasurementWindow, window2: &MeasurementWindow) -> f64 {
        let quality_diff = (window1.quality.signal_to_noise - window2.quality.signal_to_noise).abs();
        let max_snr = window1.quality.signal_to_noise.max(window2.quality.signal_to_noise);
        
        if max_snr == 0.0 {
            0.0
        } else {
            1.0 - (quality_diff / max_snr).min(1.0)
        }
    }

    /// Validate co-presence between two measurement windows
    pub fn validate_copresence(&self, window1: &MeasurementWindow, window2: &MeasurementWindow) -> bool {
        let correlation = self.calculate_correlation(window1, window2);
        let sync_score = self.calculate_sync_score(window1, window2);
        let proximity = self.calculate_proximity(window1, window2);

        correlation >= self.config.min_correlation &&
        sync_score >= 0.9 &&
        proximity >= self.config.min_proximity
    }

    /// Start collecting measurements
    pub fn start_collection(&mut self) -> Result<()> {
        let sensor_config = SensorConfig::default();
        for sensor in &mut self.sensors {
            sensor.start(&sensor_config)?;
        }

        // Collect multiple measurement windows
        for _ in 0..3 {
            // Create measurement window
            let mut measurements = Vec::new();
            let mut total_quality = EntropyQuality::default();
            let mut count = 0;

            // Collect measurements from all sensors
            for sensor in &mut self.sensors {
                let mut buffer = vec![0u8; 32];
                sensor.fill_entropy(&mut buffer)?;
                
                // Convert bytes to f64 measurements
                let sensor_measurements: Vec<f64> = buffer.iter()
                    .map(|&b| b as f64 / 255.0)
                    .collect();
                
                measurements.extend(sensor_measurements);

                // Update quality metrics
                if let Ok(quality) = sensor.quality() {
                    total_quality.shannon_entropy += quality.shannon_entropy;
                    total_quality.signal_to_noise += quality.signal_to_noise;
                    total_quality.temporal_consistency += quality.temporal_consistency;
                    count += 1;
                }
            }

            // Average quality metrics
            if count > 0 {
                total_quality.shannon_entropy /= count as f64;
                total_quality.signal_to_noise /= count as f64;
                total_quality.temporal_consistency /= count as f64;
                total_quality.sample_rate = sensor_config.sample_rate;
            }

            // Create and store measurement window
            let window = MeasurementWindow {
                start_time: SystemTime::now(),
                duration: self.config.window_size,
                measurements,
                quality: total_quality,
            };
            self.measurement_windows.push(window);

            // Wait for next window
            std::thread::sleep(self.config.window_size);
        }

        Ok(())
    }

    /// Stop collecting measurements
    pub fn stop_collection(&mut self) -> Result<()> {
        for sensor in &mut self.sensors {
            sensor.stop()?;
        }
        Ok(())
    }

    /// Get current measurement window
    pub fn get_current_window(&self) -> Option<MeasurementWindow> {
        self.measurement_windows.last().cloned()
    }

    /// Get recent measurement windows
    pub fn recent_windows(&self, count: usize) -> Vec<MeasurementWindow> {
        self.measurement_windows
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entropy::sensor::{Accelerometer, Barometer};

    #[test]
    fn test_copresence_validation() -> Result<()> {
        let config = CoPresenceConfig::default();
        let mut validator = CoPresenceValidator::new(config);

        // Add sensors
        validator.add_sensor(Accelerometer::new());
        validator.add_sensor(Barometer::new());

        // Start collection
        validator.start_collection()?;

        // Create test windows
        let window1 = MeasurementWindow {
            start_time: SystemTime::now(),
            duration: Duration::from_secs(1),
            measurements: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            quality: EntropyQuality {
                shannon_entropy: 1.0,
                sample_rate: 100.0,
                signal_to_noise: 10.0,
                temporal_consistency: 1.0,
            },
        };

        let window2 = MeasurementWindow {
            start_time: SystemTime::now(),
            duration: Duration::from_secs(1),
            measurements: vec![1.1, 2.1, 3.1, 4.1, 5.1],
            quality: EntropyQuality {
                shannon_entropy: 1.0,
                sample_rate: 100.0,
                signal_to_noise: 9.8,
                temporal_consistency: 1.0,
            },
        };

        // Test correlation
        let correlation = validator.calculate_correlation(&window1, &window2);
        assert!(correlation > 0.9, "High correlation expected");

        // Test sync score
        let sync = validator.calculate_sync_score(&window1, &window2);
        assert!(sync > 0.9, "High sync score expected");

        // Test proximity
        let proximity = validator.calculate_proximity(&window1, &window2);
        assert!(proximity > 0.9, "High proximity expected");

        // Test overall validation
        assert!(validator.validate_copresence(&window1, &window2));

        validator.stop_collection()?;
        Ok(())
    }
}
