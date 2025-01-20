use fskc::{
    Result,
    Accelerometer, Barometer, EntropyQuality,
    pairlet::{CoPresenceValidator, CoPresenceConfig, MeasurementWindow},
};
use std::time::{Duration, SystemTime};
use std::thread;

fn main() -> Result<()> {
    // Configure co-presence validation
    let config = CoPresenceConfig {
        min_correlation: 0.7,
        max_time_diff: Duration::from_millis(200),
        min_proximity: 0.8,
        window_size: Duration::from_secs(1),
    };

    // Create two validators to simulate two devices
    let mut validator1 = CoPresenceValidator::new(config.clone());
    let mut validator2 = CoPresenceValidator::new(config);

    // Add sensors to both validators
    validator1.add_sensor(Accelerometer::new());
    validator1.add_sensor(Barometer::new());
    validator2.add_sensor(Accelerometer::new());
    validator2.add_sensor(Barometer::new());

    // Start collecting measurements
    println!("Starting measurement collection...");
    validator1.start_collection()?;
    validator2.start_collection()?;

    // Simulate some time passing to collect measurements
    thread::sleep(Duration::from_secs(2));

    // Create measurement windows with slightly different but correlated data
    let window1 = MeasurementWindow {
        start_time: SystemTime::now(),
        duration: Duration::from_secs(1),
        measurements: vec![1.0, 1.2, 1.4, 1.6, 1.8],
        quality: EntropyQuality {
            shannon_entropy: 6.37,
            sample_rate: 200.0,
            signal_to_noise: 14.01,
            temporal_consistency: 1.0,
        },
    };

    let window2 = MeasurementWindow {
        start_time: SystemTime::now(),
        duration: Duration::from_secs(1),
        measurements: vec![1.1, 1.3, 1.5, 1.7, 1.9],
        quality: EntropyQuality {
            shannon_entropy: 6.35,
            sample_rate: 200.0,
            signal_to_noise: 13.98,
            temporal_consistency: 1.0,
        },
    };

    // Calculate and display metrics
    let correlation = validator1.calculate_correlation(&window1, &window2);
    let sync_score = validator1.calculate_sync_score(&window1, &window2);
    let proximity = validator1.calculate_proximity(&window1, &window2);

    println!("\nCo-presence Validation Metrics:");
    println!("Correlation: {:.2}", correlation);
    println!("Temporal Sync: {:.2}", sync_score);
    println!("Proximity: {:.2}", proximity);

    // Validate co-presence
    let is_copresent = validator1.validate_copresence(&window1, &window2);
    println!("\nCo-presence Validation Result: {}", 
        if is_copresent { "VALIDATED ✓" } else { "FAILED ✗" });

    // Stop collection
    validator1.stop_collection()?;
    validator2.stop_collection()?;

    Ok(())
}
