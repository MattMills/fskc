use fskc::{
    Result, EntropySource,
    entropy::{
        sensor::EntropyQuality,
        ios_sensor::{SensorReading, IosSensorEntropy},
    },
};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<()> {

    // Create simulated sensor readings
    let mut ios = IosSensorEntropy::new(100.0);
    
    println!("Starting iOS sensor entropy collection...\n");

    // Simulate sensor readings over time
    for i in 0..5 {
        // Create accelerometer reading
        let accel_reading = SensorReading {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: vec![i as f64 * 0.1, (i + 1) as f64 * 0.1, (i + 2) as f64 * 0.1],
            quality: EntropyQuality {
                shannon_entropy: 0.8 + (i as f64 * 0.02),
                sample_rate: 100.0,
                signal_to_noise: 10.0,
                temporal_consistency: 0.9,
            },
        };

        // Create barometer reading
        let baro_reading = SensorReading {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: vec![1013.25 + (i as f64 * 0.1)],
            quality: EntropyQuality {
                shannon_entropy: 0.85 + (i as f64 * 0.01),
                sample_rate: 100.0,
                signal_to_noise: 12.0,
                temporal_consistency: 0.95,
            },
        };

        // Add readings
        ios.add_accelerometer_reading(accel_reading)?;
        ios.add_barometer_reading(baro_reading)?;

        println!("Reading {}:", i + 1);
        println!("  Accelerometer: [{:.2}, {:.2}, {:.2}]",
            i as f64 * 0.1, (i + 1) as f64 * 0.1, (i + 2) as f64 * 0.1);
        println!("  Barometer: {:.2} hPa", 1013.25 + (i as f64 * 0.1));

        // Generate entropy
        let mut entropy_bytes = vec![0u8; 16];
        ios.fill_bytes(&mut entropy_bytes)?;

        print!("  Entropy: ");
        for byte in entropy_bytes.iter() {
            print!("{:02x} ", byte);
        }
        println!("\n");

        // Small delay between readings
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    println!("\nSensor Entropy Collection Complete");

    Ok(())
}
