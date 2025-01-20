use fskc::{
    Result,
    Accelerometer, Barometer,
    pairlet::{
        CoPresenceValidator, ContextConfig, ContextManager,
        KeyGenConfig, KeyGenerator,
    },
};
use std::time::Duration;
use std::thread;

fn main() -> Result<()> {
    // Configure context establishment
    let context_config = ContextConfig {
        min_quality: 0.8,
        required_windows: 3,
        max_age: Duration::from_secs(60),
    };

    // Configure key generation
    let keygen_config = KeyGenConfig {
        key_length: 32,       // 256-bit keys
        min_quality: 0.8,     // High quality requirement
        hash_iterations: 10000, // Strong KDF
    };

    // Create validators and context managers for two devices
    let mut manager1 = ContextManager::new(
        context_config.clone(),
        CoPresenceValidator::new(Default::default()),
    );
    let mut manager2 = ContextManager::new(
        context_config,
        CoPresenceValidator::new(Default::default()),
    );

    // Add sensors to both devices
    manager1.validator_mut().add_sensor(Accelerometer::new());
    manager1.validator_mut().add_sensor(Barometer::new());
    manager2.validator_mut().add_sensor(Accelerometer::new());
    manager2.validator_mut().add_sensor(Barometer::new());

    // Start collecting measurements
    println!("Starting measurement collection...");
    manager1.validator_mut().start_collection()?;
    manager2.validator_mut().start_collection()?;

    // Wait for measurements to accumulate
    thread::sleep(Duration::from_secs(3));

    // Get measurement windows from device 2
    let device2_windows = manager2.validator().recent_windows(3);

    // Attempt to establish shared context
    println!("\nAttempting context establishment...");
    if let Some(context) = manager1.establish_context(&device2_windows)? {
        println!("\nContext Establishment Metrics:");
        println!("Time Window: {:?}", context.time_window);
        println!("Quality Score: {:.2}", context.quality);
        println!("Measurements: {} windows", context.measurements.len());
        println!("\nContext Establishment Result: ESTABLISHED ✓");

        // Create key generator
        let generator = KeyGenerator::new(keygen_config);

        // Generate key from shared context
        println!("\nGenerating key from shared context...");
        let key = generator.generate_key(&context)?;

        // Display key information
        println!("\nKey Generation Results:");
        println!("Key Length: {} bytes", key.key.len());
        println!("Key Quality: {:.2}", key.quality);
        println!("Generated At: {:?}", key.generated_at);
        println!("\nKey Material (first 16 bytes):");
        for byte in key.key.iter().take(16) {
            print!("{:02x} ", byte);
        }
        println!("\n\nVerification Hash:");
        for byte in &key.verification_hash {
            print!("{:02x} ", byte);
        }
        println!();

        // Verify key
        println!("\nKey Verification:");
        println!("Verification Result: {}", 
            if generator.verify_key(&key, &key.verification_hash) {
                "VALID ✓"
            } else {
                "INVALID ✗"
            }
        );
    } else {
        println!("\nContext Establishment Failed - Cannot Generate Key");
    }

    // Stop collection
    manager1.validator_mut().stop_collection()?;
    manager2.validator_mut().stop_collection()?;

    Ok(())
}
