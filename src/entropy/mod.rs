use crate::Result;
use rand::{RngCore, Error as RngError, CryptoRng};
use std::sync::{Arc, Mutex};
use getrandom::Error as GetRandomError;

pub mod sensor;
use sensor::{Sensor, SensorConfig, EntropyQuality};

/// Represents a source of entropy
pub trait EntropySource: Send + Sync {
    /// Fills the provided buffer with entropy
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<()>;
    
    /// Returns a description of the entropy source
    fn description(&self) -> &str;
}

/// Standard RNG-based entropy source
pub struct RngEntropy<R: RngCore> {
    rng: R,
    description: String,
}

impl<R: RngCore> RngEntropy<R> {
    pub fn new(rng: R, description: impl Into<String>) -> Self {
        Self {
            rng,
            description: description.into(),
        }
    }
}

impl<R: RngCore + Send + Sync> EntropySource for RngEntropy<R> {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<()> {
        self.rng.fill_bytes(dest);
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Physical measurement-based entropy source
pub struct PhysicalEntropy {
    data: Vec<u8>,
    position: usize,
    description: String,
}

impl PhysicalEntropy {
    pub fn new(data: Vec<u8>, description: impl Into<String>) -> Self {
        Self {
            data,
            position: 0,
            description: description.into(),
        }
    }

    /// Creates a LIGO gravitational wave data source
    pub fn from_ligo_data(data: Vec<u8>) -> Self {
        Self::new(data, "LIGO Gravitational Wave Data")
    }

    /// Creates a stellar parallax measurement source
    pub fn from_stellar_parallax(data: Vec<u8>) -> Self {
        Self::new(data, "Stellar Parallax Measurements")
    }
}

impl EntropySource for PhysicalEntropy {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<()> {
        for byte in dest.iter_mut() {
            if self.position >= self.data.len() {
                self.position = 0; // Wrap around
            }
            *byte = self.data[self.position];
            self.position += 1;
        }
        Ok(())
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Combines multiple entropy sources including physical sensors
pub struct CombinedEntropy {
    sources: Vec<Box<dyn EntropySource>>,
    sensors: Vec<Box<dyn Sensor>>,
    buffer: Vec<u8>,
    position: usize,
    sensor_config: SensorConfig,
}

impl CombinedEntropy {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            sensors: Vec::new(),
            buffer: Vec::new(),
            position: 0,
            sensor_config: SensorConfig::default(),
        }
    }

    /// Add a sensor to the entropy pool
    pub fn add_sensor<S: Sensor + 'static>(&mut self, mut sensor: S) -> Result<()> {
        sensor.start(&self.sensor_config)?;
        self.sensors.push(Box::new(sensor));
        Ok(())
    }

    /// Set sensor configuration
    pub fn set_sensor_config(&mut self, config: SensorConfig) -> Result<()> {
        self.sensor_config = config;
        // Update all sensors with new config
        for sensor in &mut self.sensors {
            sensor.start(&self.sensor_config)?;
        }
        Ok(())
    }

    /// Get entropy quality metrics from all sensors
    pub fn sensor_quality(&self) -> Vec<(String, EntropyQuality)> {
        self.sensors
            .iter()
            .filter_map(|s| {
                s.quality()
                    .ok()
                    .map(|q| (s.description().to_string(), q))
            })
            .collect()
    }

    /// Adds an entropy source to the combination
    pub fn add_source<S: EntropySource + 'static>(&mut self, source: S) {
        self.sources.push(Box::new(source));
    }

    /// Lists all entropy sources
    pub fn list_sources(&self) -> Vec<&str> {
        self.sources.iter().map(|s| s.description()).collect()
    }

    fn ensure_buffer(&mut self, required: usize) -> Result<()> {
        if self.position + required > self.buffer.len() {
            // Reset buffer and position
            self.buffer.clear();
            self.position = 0;
            
            // Fill buffer with new entropy
            let mut new_buffer = vec![0u8; 1024.max(required)];
            self.fill_entropy(&mut new_buffer)?;
            self.buffer = new_buffer;
        }
        Ok(())
    }

    fn fill_entropy(&mut self, dest: &mut [u8]) -> Result<()> {
        // Initialize buffer with zeros
        for byte in dest.iter_mut() {
            *byte = 0;
        }

        // Collect entropy from standard sources
        for source in &mut self.sources {
            let mut source_bytes = vec![0u8; dest.len()];
            source.fill_bytes(&mut source_bytes)?;
            
            // XOR with existing buffer
            for (buf_byte, src_byte) in dest.iter_mut().zip(source_bytes.iter()) {
                *buf_byte ^= src_byte;
            }
        }

        // Collect entropy from sensors
        for sensor in &mut self.sensors {
            let mut sensor_bytes = vec![0u8; dest.len()];
            sensor.fill_entropy(&mut sensor_bytes)?;
            
            // XOR with existing buffer
            for (buf_byte, src_byte) in dest.iter_mut().zip(sensor_bytes.iter()) {
                *buf_byte ^= src_byte;
            }
        }

        Ok(())
    }
}

impl RngCore for CombinedEntropy {
    fn next_u32(&mut self) -> u32 {
        let mut bytes = [0u8; 4];
        self.fill_bytes(&mut bytes);
        u32::from_le_bytes(bytes)
    }

    fn next_u64(&mut self) -> u64 {
        let mut bytes = [0u8; 8];
        self.fill_bytes(&mut bytes);
        u64::from_le_bytes(bytes)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        if let Err(e) = self.try_fill_bytes(dest) {
            panic!("Entropy generation failed: {}", e);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> std::result::Result<(), RngError> {
        if let Err(_) = self.ensure_buffer(dest.len()) {
            return Err(RngError::from(GetRandomError::UNSUPPORTED));
        }

        // Copy from buffer to destination
        let available = self.buffer.len() - self.position;
        let needed = dest.len();
        
        if needed <= available {
            dest.copy_from_slice(&self.buffer[self.position..self.position + needed]);
            self.position += needed;
        } else {
            // Fill what we can from current buffer
            dest[..available].copy_from_slice(&self.buffer[self.position..]);
            
            // Generate new entropy for the rest
            self.buffer.clear();
            self.position = 0;
            let mut new_buffer = vec![0u8; 1024.max(needed - available)];
            if let Err(_) = self.fill_entropy(&mut new_buffer) {
                return Err(RngError::from(GetRandomError::UNSUPPORTED));
            }
            
            dest[available..].copy_from_slice(&new_buffer[..needed - available]);
            self.buffer = new_buffer;
            self.position = needed - available;
        }
        Ok(())
    }
}

// Mark CombinedEntropy as cryptographically secure
impl CryptoRng for CombinedEntropy {}

/// Builder for creating entropy configurations
pub struct EntropyBuilder {
    combined: CombinedEntropy,
    sensor_config: SensorConfig,
}

impl EntropyBuilder {
    pub fn new() -> Self {
        Self {
            combined: CombinedEntropy::new(),
            sensor_config: SensorConfig::default(),
        }
    }

    /// Configure sensor parameters
    pub fn with_sensor_config(mut self, config: SensorConfig) -> Self {
        self.sensor_config = config;
        self
    }

    /// Add an accelerometer sensor
    pub fn add_accelerometer(mut self) -> Self {
        let accel = sensor::Accelerometer::new();
        if let Ok(()) = self.combined.add_sensor(accel) {
            // Sensor added successfully
        }
        self
    }

    /// Add a barometer sensor
    pub fn add_barometer(mut self) -> Self {
        let baro = sensor::Barometer::new();
        if let Ok(()) = self.combined.add_sensor(baro) {
            // Sensor added successfully
        }
        self
    }

    /// Add a custom sensor
    pub fn add_sensor<S: Sensor + 'static>(mut self, sensor: S) -> Self {
        if let Ok(()) = self.combined.add_sensor(sensor) {
            // Sensor added successfully
        }
        self
    }

    /// Adds a standard RNG source
    pub fn add_rng<R: RngCore + Send + Sync + 'static>(
        mut self,
        rng: R,
        description: impl Into<String>,
    ) -> Self {
        self.combined.add_source(RngEntropy::new(rng, description));
        self
    }

    /// Adds LIGO gravitational wave data as an entropy source
    pub fn add_ligo_data(mut self, data: Vec<u8>) -> Self {
        self.combined.add_source(PhysicalEntropy::from_ligo_data(data));
        self
    }

    /// Adds stellar parallax measurements as an entropy source
    pub fn add_stellar_parallax(mut self, data: Vec<u8>) -> Self {
        self.combined.add_source(PhysicalEntropy::from_stellar_parallax(data));
        self
    }

    /// Adds a custom physical entropy source
    pub fn add_physical_source(
        mut self,
        data: Vec<u8>,
        description: impl Into<String>,
    ) -> Self {
        self.combined.add_source(PhysicalEntropy::new(data, description));
        self
    }

    /// Builds the combined entropy source
    pub fn build(self) -> Arc<Mutex<CombinedEntropy>> {
        Arc::new(Mutex::new(self.combined))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_entropy_generation() {
        // Create entropy sources
        let mut entropy = CombinedEntropy::new();
        
        // Create RNGs
        let rng1 = ChaCha20Rng::seed_from_u64(12345);
        let rng2 = ChaCha20Rng::seed_from_u64(67890);

        // Add RNG sources
        entropy.add_source(RngEntropy::new(rng1, "ChaCha20 RNG 1"));
        entropy.add_source(RngEntropy::new(rng2, "ChaCha20 RNG 2"));
        
        // Add physical sources
        entropy.add_source(PhysicalEntropy::from_ligo_data(vec![0x42; 1024]));
        entropy.add_source(PhysicalEntropy::from_stellar_parallax(vec![0x17; 1024]));

        // Test entropy generation using RngCore trait
        let mut bytes1 = vec![0u8; 32];
        let mut bytes2 = vec![0u8; 32];
        
        RngCore::fill_bytes(&mut entropy, &mut bytes1);
        RngCore::fill_bytes(&mut entropy, &mut bytes2);
        
        // Verify different values
        assert_ne!(bytes1, bytes2);
        
        // Verify sources
        let sources = entropy.list_sources();
        assert_eq!(sources.len(), 4);
        assert!(sources.contains(&"ChaCha20 RNG 1"));
        assert!(sources.contains(&"LIGO Gravitational Wave Data"));
    }

    #[test]
    fn test_rng_interface() {
        // Create entropy source
        let mut entropy = CombinedEntropy::new();
        let rng = ChaCha20Rng::seed_from_u64(12345);
        entropy.add_source(RngEntropy::new(rng, "ChaCha20"));

        // Test RNG interface
        let mut bytes = vec![0u8; 100];
        let mut bytes2 = vec![0u8; 100];

        let _u32 = entropy.next_u32();
        let _u64 = entropy.next_u64();
        entropy.fill_bytes(&mut bytes);
        entropy.fill_bytes(&mut bytes2);

        // Verify different values
        assert_ne!(bytes, bytes2);
    }
}
