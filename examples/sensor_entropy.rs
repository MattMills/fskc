use fskc::{
    Result,
    entropy::{
        sensor::SensorConfig,
        EntropyBuilder,
    },
};
use rand_chacha::ChaCha20Rng;
use rand::{SeedableRng, RngCore};
use std::time::Duration;

fn main() -> Result<()> {
    // Configure sensors
    let sensor_config = SensorConfig {
        sample_rate: 200.0,  // 200 Hz
        precision: 16,       // 16-bit samples
        min_quality: 0.8,    // 80% minimum entropy quality
        window: Duration::from_millis(100),
    };

    // Create entropy pool with sensors and standard RNG
    let entropy = EntropyBuilder::new()
        .with_sensor_config(sensor_config)
        .add_accelerometer()
        .add_barometer()
        .add_rng(
            ChaCha20Rng::seed_from_u64(12345),
            "ChaCha20 Base RNG"
        )
        .build();

    // Get entropy pool
    let mut entropy = entropy.lock().unwrap();

    // Check sensor quality
    println!("\nSensor Quality Metrics:");
    for (sensor, quality) in entropy.sensor_quality() {
        println!(
            "{}: Shannon Entropy={:.2}, SNR={:.2}, Temporal Consistency={:.2}",
            sensor,
            quality.shannon_entropy,
            quality.signal_to_noise,
            quality.temporal_consistency
        );
    }

    // Generate some random bytes using the combined entropy
    let mut random_bytes = vec![0u8; 32];
    (&mut *entropy).fill_bytes(&mut random_bytes);

    println!("\nGenerated Random Bytes:");
    for chunk in random_bytes.chunks(8) {
        for byte in chunk {
            print!("{:02x} ", byte);
        }
        println!();
    }

    // Calculate basic statistics
    let mean = random_bytes.iter().map(|&x| x as f64).sum::<f64>() / random_bytes.len() as f64;
    let variance = random_bytes.iter()
        .map(|&x| {
            let diff = x as f64 - mean;
            diff * diff
        })
        .sum::<f64>() / random_bytes.len() as f64;

    println!("\nBasic Statistics:");
    println!("Mean: {:.2}", mean);
    println!("Standard Deviation: {:.2}", variance.sqrt());

    Ok(())
}
