use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::Result;
use super::{EntropySource, EntropyQuality};

/// Represents a sensor reading with timestamp
#[derive(Debug, Clone)]
pub struct SensorReading {
    /// Timestamp of the reading
    pub timestamp: u64,
    /// Raw sensor data
    pub data: Vec<f64>,
    /// Quality metrics for this reading
    pub quality: EntropyQuality,
}

/// Ring buffer for temporal sensor data
#[derive(Debug)]
pub struct RingBuffer<T> {
    /// Buffer capacity
    capacity: usize,
    /// Stored items
    items: VecDeque<T>,
}

impl<T> RingBuffer<T> {
    /// Create new ring buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: VecDeque::with_capacity(capacity),
        }
    }

    /// Add item to buffer, removing oldest if at capacity
    pub fn push(&mut self, item: T) {
        if self.items.len() >= self.capacity {
            self.items.pop_front();
        }
        self.items.push_back(item);
    }

    /// Get slice of most recent items
    pub fn recent(&mut self, count: usize) -> Vec<T> 
    where T: Clone {
        let start = self.items.len().saturating_sub(count);
        self.items.make_contiguous()[start..].to_vec()
    }
}

/// Accelerometer-based entropy source
pub struct AccelerometerSource {
    /// Temporal buffer of readings
    buffer: RingBuffer<SensorReading>,
    /// Sample rate in Hz
    sample_rate: f64,
    /// Current quality metrics
    quality: EntropyQuality,
}

impl AccelerometerSource {
    /// Create new accelerometer source
    pub fn new(sample_rate: f64) -> Self {
        Self {
            buffer: RingBuffer::new(1000),
            sample_rate,
            quality: EntropyQuality::default(),
        }
    }

    /// Add new sensor reading
    pub fn add_reading(&mut self, reading: SensorReading) {
        self.buffer.push(reading);
        self.update_quality();
    }

    /// Update quality metrics based on recent readings
    fn update_quality(&mut self) {
        // Calculate Shannon entropy from recent readings
        let recent = self.buffer.recent(100);
        if recent.is_empty() {
            return;
        }

        // Calculate entropy metrics
        let mut total_entropy = 0.0;
        let mut total_snr = 0.0;
        let mut total_consistency = 0.0;
        let mut count = 0;

        for reading in recent {
            total_entropy += reading.quality.shannon_entropy;
            total_snr += reading.quality.signal_to_noise;
            total_consistency += reading.quality.temporal_consistency;
            count += 1;
        }

        if count > 0 {
            self.quality = EntropyQuality {
                shannon_entropy: total_entropy / count as f64,
                sample_rate: self.sample_rate,
                signal_to_noise: total_snr / count as f64,
                temporal_consistency: total_consistency / count as f64,
            };
        }
    }
}

impl EntropySource for AccelerometerSource {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<()> {
        // Get recent readings
        let recent = self.buffer.recent(dest.len());
        if recent.is_empty() {
            return Err(crate::FskcError::Custom(
                "No accelerometer readings available".into()
            ));
        }

        // Convert readings to bytes
        for (i, reading) in recent.iter().enumerate() {
            if i >= dest.len() {
                break;
            }
            // Use first data point, scaled to byte range
            let byte_val = (reading.data[0] * 255.0) as u8;
            dest[i] = byte_val;
        }

        Ok(())
    }

    fn description(&self) -> &str {
        "iOS Accelerometer Entropy Source"
    }
}

/// Barometer-based entropy source
pub struct BarometerSource {
    /// Temporal buffer of readings
    buffer: RingBuffer<SensorReading>,
    /// Sample rate in Hz
    sample_rate: f64,
    /// Current quality metrics
    quality: EntropyQuality,
}

impl BarometerSource {
    /// Create new barometer source
    pub fn new(sample_rate: f64) -> Self {
        Self {
            buffer: RingBuffer::new(1000),
            sample_rate,
            quality: EntropyQuality::default(),
        }
    }

    /// Add new sensor reading
    pub fn add_reading(&mut self, reading: SensorReading) {
        self.buffer.push(reading);
        self.update_quality();
    }

    /// Update quality metrics based on recent readings
    fn update_quality(&mut self) {
        // Calculate Shannon entropy from recent readings
        let recent = self.buffer.recent(100);
        if recent.is_empty() {
            return;
        }

        // Calculate entropy metrics
        let mut total_entropy = 0.0;
        let mut total_snr = 0.0;
        let mut total_consistency = 0.0;
        let mut count = 0;

        for reading in recent {
            total_entropy += reading.quality.shannon_entropy;
            total_snr += reading.quality.signal_to_noise;
            total_consistency += reading.quality.temporal_consistency;
            count += 1;
        }

        if count > 0 {
            self.quality = EntropyQuality {
                shannon_entropy: total_entropy / count as f64,
                sample_rate: self.sample_rate,
                signal_to_noise: total_snr / count as f64,
                temporal_consistency: total_consistency / count as f64,
            };
        }
    }
}

impl EntropySource for BarometerSource {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<()> {
        // Get recent readings
        let recent = self.buffer.recent(dest.len());
        if recent.is_empty() {
            return Err(crate::FskcError::Custom(
                "No barometer readings available".into()
            ));
        }

        // Convert readings to bytes
        for (i, reading) in recent.iter().enumerate() {
            if i >= dest.len() {
                break;
            }
            // Use first data point, scaled to byte range
            let byte_val = (reading.data[0] * 255.0) as u8;
            dest[i] = byte_val;
        }

        Ok(())
    }

    fn description(&self) -> &str {
        "iOS Barometer Entropy Source"
    }
}

/// Combined iOS sensor entropy source
pub struct IosSensorEntropy {
    /// Accelerometer source
    accelerometer: Arc<Mutex<AccelerometerSource>>,
    /// Barometer source
    barometer: Arc<Mutex<BarometerSource>>,
    /// Temporal buffer for combined readings
    temporal_buffer: RingBuffer<SensorReading>,
    /// Combined quality metrics
    quality: EntropyQuality,
}

impl IosSensorEntropy {
    /// Create new iOS sensor entropy source
    pub fn new(sample_rate: f64) -> Self {
        Self {
            accelerometer: Arc::new(Mutex::new(AccelerometerSource::new(sample_rate))),
            barometer: Arc::new(Mutex::new(BarometerSource::new(sample_rate))),
            temporal_buffer: RingBuffer::new(1000),
            quality: EntropyQuality::default(),
        }
    }

    /// Add accelerometer reading
    pub fn add_accelerometer_reading(&mut self, reading: SensorReading) -> Result<()> {
        self.accelerometer.lock().unwrap().add_reading(reading.clone());
        self.temporal_buffer.push(reading);
        self.update_quality();
        Ok(())
    }

    /// Add barometer reading
    pub fn add_barometer_reading(&mut self, reading: SensorReading) -> Result<()> {
        self.barometer.lock().unwrap().add_reading(reading.clone());
        self.temporal_buffer.push(reading);
        self.update_quality();
        Ok(())
    }

    /// Get temporal slice of readings
    pub fn get_temporal_slice(&mut self, count: usize) -> Vec<SensorReading> {
        self.temporal_buffer.recent(count)
    }

    /// Get current quality metrics
    pub fn quality(&self) -> EntropyQuality {
        self.quality
    }

    /// Update combined quality metrics
    fn update_quality(&mut self) {
        let accel_quality = self.accelerometer.lock().unwrap().quality;
        let baro_quality = self.barometer.lock().unwrap().quality;

        // Average quality metrics
        self.quality = EntropyQuality {
            shannon_entropy: (accel_quality.shannon_entropy + baro_quality.shannon_entropy) / 2.0,
            sample_rate: accel_quality.sample_rate,
            signal_to_noise: (accel_quality.signal_to_noise + baro_quality.signal_to_noise) / 2.0,
            temporal_consistency: (accel_quality.temporal_consistency + baro_quality.temporal_consistency) / 2.0,
        };
    }
}

impl EntropySource for IosSensorEntropy {
    fn fill_bytes(&mut self, dest: &mut [u8]) -> Result<()> {
        // Split destination between sources
        let mid = dest.len() / 2;
        let (accel_dest, baro_dest) = dest.split_at_mut(mid);

        // Fill from both sources
        self.accelerometer.lock().unwrap().fill_bytes(accel_dest)?;
        self.barometer.lock().unwrap().fill_bytes(baro_dest)?;

        Ok(())
    }

    fn description(&self) -> &str {
        "Combined iOS Sensor Entropy Source"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_reading(data: Vec<f64>) -> SensorReading {
        SensorReading {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data,
            quality: EntropyQuality {
                shannon_entropy: 0.8,
                sample_rate: 100.0,
                signal_to_noise: 10.0,
                temporal_consistency: 0.9,
            },
        }
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);
        
        // Add items
        buffer.push(1);
        buffer.push(2);
        buffer.push(3);
        assert_eq!(buffer.recent(3), vec![1, 2, 3]);

        // Add item beyond capacity
        buffer.push(4);
        assert_eq!(buffer.recent(3), vec![2, 3, 4]);
    }

    #[test]
    fn test_accelerometer_source() -> Result<()> {
        let mut source = AccelerometerSource::new(100.0);

        // Add readings
        for i in 0..5 {
            source.add_reading(create_test_reading(vec![i as f64]));
        }

        // Test entropy generation
        let mut dest = vec![0u8; 3];
        source.fill_bytes(&mut dest)?;
        assert!(!dest.iter().all(|&x| x == 0));

        Ok(())
    }

    #[test]
    fn test_combined_entropy() -> Result<()> {
        let mut entropy = IosSensorEntropy::new(100.0);

        // Add readings to both sources
        for i in 0..5 {
            entropy.add_accelerometer_reading(create_test_reading(vec![i as f64]))?;
            entropy.add_barometer_reading(create_test_reading(vec![i as f64 + 0.5]))?;
        }

        // Test entropy generation
        let mut dest = vec![0u8; 6];
        entropy.fill_bytes(&mut dest)?;
        assert!(!dest.iter().all(|&x| x == 0));

        Ok(())
    }
}
