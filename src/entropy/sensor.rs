use std::time::Duration;
use crate::Result;

/// Represents the quality of entropy from a sensor
#[derive(Debug, Clone, Copy, Default)]
pub struct EntropyQuality {
    /// Shannon entropy estimate (0.0 to 1.0)
    pub shannon_entropy: f64,
    /// Sample rate in Hz
    pub sample_rate: f64,
    /// Signal-to-noise ratio estimate
    pub signal_to_noise: f64,
    /// Temporal consistency score (0.0 to 1.0)
    pub temporal_consistency: f64,
}

/// Configuration for sensor sampling
#[derive(Debug, Clone)]
pub struct SensorConfig {
    /// Desired sample rate in Hz
    pub sample_rate: f64,
    /// Sample precision in bits
    pub precision: u8,
    /// Minimum required entropy quality
    pub min_quality: f64,
    /// Sample window duration
    pub window: Duration,
}

impl Default for SensorConfig {
    fn default() -> Self {
        Self {
            sample_rate: 100.0,  // 100 Hz default
            precision: 16,       // 16-bit precision
            min_quality: 0.7,    // 70% minimum entropy quality
            window: Duration::from_secs(1),
        }
    }
}

/// Represents a physical sensor that can provide entropy
pub trait Sensor: Send + Sync {
    /// Check if the sensor hardware is available
    fn check_hardware(&self) -> bool;
    
    /// Start collecting sensor data
    fn start(&mut self, config: &SensorConfig) -> Result<()>;
    
    /// Stop collecting sensor data
    fn stop(&mut self) -> Result<()>;
    
    /// Get the current entropy quality metrics
    fn quality(&self) -> Result<EntropyQuality>;
    
    /// Fill buffer with entropy from sensor
    fn fill_entropy(&mut self, buffer: &mut [u8]) -> Result<()>;
    
    /// Get sensor description
    fn description(&self) -> &str;
    
    /// Get current configuration
    fn config(&self) -> &SensorConfig;
}

/// Accelerometer sensor implementation
pub struct Accelerometer {
    config: SensorConfig,
    running: bool,
    description: String,
    // Internal state
    samples: Vec<[f64; 3]>,
    last_quality: EntropyQuality,
}

impl Accelerometer {
    pub fn new() -> Self {
        // Generate some simulated accelerometer data
        let mut samples = Vec::new();
        for i in 0..100 {
            let t = i as f64 * 0.01;
            samples.push([
                (t * 2.0 * std::f64::consts::PI).sin() * 0.5,  // X axis
                (t * 3.0 * std::f64::consts::PI).cos() * 0.3,  // Y axis
                0.1 * (t * 5.0).sin(),                         // Z axis
            ]);
        }

        Self {
            config: SensorConfig::default(),
            running: false,
            description: "3-Axis Accelerometer".into(),
            samples,
            last_quality: EntropyQuality {
                shannon_entropy: 0.0,
                sample_rate: 0.0,
                signal_to_noise: 0.0,
                temporal_consistency: 0.0,
            },
        }
    }

    /// Calculate Shannon entropy from samples
    fn calculate_shannon_entropy(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        // Calculate histogram of values
        let mut counts = vec![0.0; 256];
        let mut total = 0.0;

        for sample in &self.samples {
            for &axis in sample {
                // Map -1.0..1.0 to 0..255
                let bin = ((axis + 1.0) * 127.5) as usize;
                let bin = bin.clamp(0, 255);
                counts[bin] += 1.0;
                total += 1.0;
            }
        }

        // Calculate Shannon entropy
        let mut entropy = 0.0;
        for count in counts {
            if count > 0.0 {
                let p: f64 = count / total;
                entropy -= p * f64::log2(p);
            }
        }

        entropy
    }

    /// Update quality metrics based on recent samples
    fn update_quality(&mut self) {
        if self.samples.is_empty() {
            return;
        }

        // Calculate signal-to-noise ratio across all axes
        let mut signal = 0.0;
        let mut noise = 0.0;
        for window in self.samples.windows(2) {
            for axis in 0..3 {
                let diff = window[1][axis] - window[0][axis];
                signal += window[0][axis].abs();
                noise += diff.abs();
            }
        }
        let snr = if noise == 0.0 { 0.0 } else { signal / noise };

        // Calculate temporal consistency
        let mut consistency = 0.0;
        if self.samples.len() > 1 {
            let mut diffs = Vec::new();
            for window in self.samples.windows(2) {
                for axis in 0..3 {
                    diffs.push((window[1][axis] - window[0][axis]).abs());
                }
            }
            let mean_diff = diffs.iter().sum::<f64>() / diffs.len() as f64;
            let var_diff = diffs.iter()
                .map(|&d| (d - mean_diff).powi(2))
                .sum::<f64>() / diffs.len() as f64;
            consistency = 1.0 / (1.0 + var_diff);
        }

        self.last_quality = EntropyQuality {
            shannon_entropy: self.calculate_shannon_entropy(),
            sample_rate: self.config.sample_rate,
            signal_to_noise: snr,
            temporal_consistency: consistency,
        };
    }
}

impl Sensor for Accelerometer {
    fn check_hardware(&self) -> bool {
        // In a real implementation, this would check for actual hardware
        // For now, return false since we're using simulated data
        false
    }

    fn start(&mut self, config: &SensorConfig) -> Result<()> {
        if !self.check_hardware() {
            return Err(crate::FskcError::EntropyError("Hardware not available".into()));
        }
        self.config = config.clone();
        self.running = true;
        self.update_quality();
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn quality(&self) -> Result<EntropyQuality> {
        Ok(self.last_quality)
    }

    fn fill_entropy(&mut self, buffer: &mut [u8]) -> Result<()> {
        if !self.running {
            return Err(crate::FskcError::EntropyError("Sensor not running".into()));
        }

        // Simple entropy extraction for demonstration
        // In practice, would use proper whitening and extraction methods
        for (i, byte) in buffer.iter_mut().enumerate() {
            if let Some(sample) = self.samples.get(i % self.samples.len()) {
                let mixed = sample[0].abs() + sample[1].abs() + sample[2].abs();
                *byte = (mixed * 255.0) as u8;
            }
        }

        self.update_quality();
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn config(&self) -> &SensorConfig {
        &self.config
    }
}

/// Barometer sensor implementation
pub struct Barometer {
    config: SensorConfig,
    running: bool,
    description: String,
    // Internal state
    samples: Vec<f64>,
    last_quality: EntropyQuality,
}

impl Barometer {
    pub fn new() -> Self {
        // Generate some simulated barometric pressure data
        let mut samples = Vec::new();
        let base_pressure = 1013.25; // Standard atmospheric pressure in hPa
        for i in 0..100 {
            let t = i as f64 * 0.01;
            // Simulate small pressure variations
            let pressure = base_pressure + 
                0.5 * (t * 2.0 * std::f64::consts::PI).sin() +  // Slow variation
                0.1 * (t * 10.0 * std::f64::consts::PI).sin();  // Fast variation
            samples.push(pressure);
        }

        Self {
            config: SensorConfig::default(),
            running: false,
            description: "Barometric Pressure Sensor".into(),
            samples,
            last_quality: EntropyQuality {
                shannon_entropy: 0.0,
                sample_rate: 0.0,
                signal_to_noise: 0.0,
                temporal_consistency: 0.0,
            },
        }
    }

    fn calculate_shannon_entropy(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }

        // Calculate histogram of values
        let mut counts = vec![0.0; 256];
        let mut total = 0.0;

        for &pressure in &self.samples {
            // Map pressure variations to 0..255
            let normalized = (pressure - 1013.25) * 10.0; // Amplify variations
            let bin = ((normalized + 128.0) as usize).clamp(0, 255);
            counts[bin] += 1.0;
            total += 1.0;
        }

        // Calculate Shannon entropy
        let mut entropy = 0.0;
        for count in counts {
            if count > 0.0 {
                let p: f64 = count / total;
                entropy -= p * f64::log2(p);
            }
        }

        entropy
    }

    fn update_quality(&mut self) {
        if self.samples.is_empty() {
            return;
        }

        // Calculate signal-to-noise ratio
        let mut signal = 0.0;
        let mut noise = 0.0;
        for window in self.samples.windows(2) {
            let diff = window[1] - window[0];
            signal += window[0].abs();
            noise += diff.abs();
        }
        let snr = if noise == 0.0 { 0.0 } else { signal / noise };

        // Calculate temporal consistency
        let mut consistency = 0.0;
        if self.samples.len() > 1 {
            let mut diffs = Vec::new();
            for window in self.samples.windows(2) {
                diffs.push((window[1] - window[0]).abs());
            }
            let mean_diff = diffs.iter().sum::<f64>() / diffs.len() as f64;
            let var_diff = diffs.iter()
                .map(|&d| (d - mean_diff).powi(2))
                .sum::<f64>() / diffs.len() as f64;
            consistency = 1.0 / (1.0 + var_diff);
        }

        self.last_quality = EntropyQuality {
            shannon_entropy: self.calculate_shannon_entropy(),
            sample_rate: self.config.sample_rate,
            signal_to_noise: snr,
            temporal_consistency: consistency,
        };
    }
}

impl Sensor for Barometer {
    fn check_hardware(&self) -> bool {
        // In a real implementation, this would check for actual hardware
        // For now, return false since we're using simulated data
        false
    }

    fn start(&mut self, config: &SensorConfig) -> Result<()> {
        if !self.check_hardware() {
            return Err(crate::FskcError::EntropyError("Hardware not available".into()));
        }
        self.config = config.clone();
        self.running = true;
        self.update_quality();
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    fn quality(&self) -> Result<EntropyQuality> {
        Ok(self.last_quality)
    }

    fn fill_entropy(&mut self, buffer: &mut [u8]) -> Result<()> {
        if !self.running {
            return Err(crate::FskcError::EntropyError("Sensor not running".into()));
        }

        // Simple entropy extraction for demonstration
        for (i, byte) in buffer.iter_mut().enumerate() {
            if let Some(sample) = self.samples.get(i % self.samples.len()) {
                *byte = (sample.abs() * 255.0) as u8;
            }
        }

        self.update_quality();
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn config(&self) -> &SensorConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accelerometer() -> Result<()> {
        let mut accel = Accelerometer::new();
        
        if !accel.check_hardware() {
            println!("Skipping accelerometer test - no hardware available");
            return Ok(());
        }

        let config = SensorConfig::default();

        // Test sensor lifecycle
        accel.start(&config)?;
        assert!(accel.running);

        // Test entropy generation
        let mut buffer = vec![0u8; 32];
        accel.fill_entropy(&mut buffer)?;

        // Test quality metrics
        let quality = accel.quality()?;
        assert!(quality.shannon_entropy >= 0.0);
        assert!(quality.shannon_entropy <= 1.0);

        accel.stop()?;
        assert!(!accel.running);

        Ok(())
    }

    #[test]
    fn test_barometer() -> Result<()> {
        let mut baro = Barometer::new();
        
        if !baro.check_hardware() {
            println!("Skipping barometer test - no hardware available");
            return Ok(());
        }

        let config = SensorConfig::default();

        // Test sensor lifecycle
        baro.start(&config)?;
        assert!(baro.running);

        // Test entropy generation
        let mut buffer = vec![0u8; 32];
        baro.fill_entropy(&mut buffer)?;

        // Test quality metrics
        let quality = baro.quality()?;
        assert!(quality.shannon_entropy >= 0.0);
        assert!(quality.shannon_entropy <= 1.0);

        baro.stop()?;
        assert!(!baro.running);

        Ok(())
    }
}
