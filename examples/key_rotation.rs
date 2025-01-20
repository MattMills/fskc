use fskc::{
    Result,
    Accelerometer, Barometer,
    pairlet::{
        CoPresenceValidator, ContextConfig, ContextManager,
        KeyGenConfig, KeyGenerator, ExchangeConfig, KeyExchange,
        RotationConfig, KeyRotation,
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

    // Configure key exchange
    let exchange_config = ExchangeConfig {
        confirmation_timeout: Duration::from_secs(30),
        confirmation_rounds: 3,
        min_key_quality: 0.8,
    };

    // Configure key rotation
    let rotation_config = RotationConfig {
        max_key_lifetime: Duration::from_secs(5),      // Short lifetime for demo
        rotation_quality_threshold: 0.8,
        max_history_size: 3,
        min_rotation_interval: Duration::from_secs(2), // Short interval for demo
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

    // Create key exchanges for both devices
    let mut exchange1 = KeyExchange::new(
        exchange_config.clone(),
        KeyGenerator::new(keygen_config.clone()),
    );
    let mut exchange2 = KeyExchange::new(
        exchange_config,
        KeyGenerator::new(keygen_config.clone()),
    );

    // Create key rotation managers
    let mut rotation1 = KeyRotation::new(
        rotation_config.clone(),
        KeyGenerator::new(keygen_config.clone()),
    );
    let mut rotation2 = KeyRotation::new(
        rotation_config,
        KeyGenerator::new(keygen_config),
    );

    // Start collecting measurements
    println!("Starting measurement collection...");
    manager1.validator_mut().start_collection()?;
    manager2.validator_mut().start_collection()?;

    // Simulate key rotation over time
    for i in 1..=3 {
        println!("\nRotation Cycle {}", i);
        println!("------------------");

        // Wait for measurements
        thread::sleep(Duration::from_secs(3));

        // Get measurement windows from device 2
        let device2_windows = manager2.validator().recent_windows(3);

        // Attempt to establish shared context
        println!("\nAttempting context establishment...");
        if let Some(context) = manager1.establish_context(&device2_windows)? {
            println!("Context Quality: {:.2}", context.quality);

            // Check if rotation is needed
            if rotation1.needs_rotation(&context) {
                println!("\nRotating keys...");

                // Rotate keys on both devices
                rotation1.rotate_key(&context, &mut exchange1)?;
                rotation2.rotate_key(&context, &mut exchange2)?;

                // Display active keys
                if let Some(key1) = rotation1.active_key() {
                    println!("\nDevice 1 Active Key:");
                    println!("Status: {:?}", key1.status);
                    println!("Quality: {:.2}", key1.key.quality);
                    println!("Activated: {:?}", key1.activated_at);
                    print!("Material (first 16 bytes): ");
                    for byte in key1.key.key.iter().take(16) {
                        print!("{:02x} ", byte);
                    }
                    println!();
                }

                if let Some(key2) = rotation2.active_key() {
                    println!("\nDevice 2 Active Key:");
                    println!("Status: {:?}", key2.status);
                    println!("Quality: {:.2}", key2.key.quality);
                    println!("Activated: {:?}", key2.activated_at);
                    print!("Material (first 16 bytes): ");
                    for byte in key2.key.key.iter().take(16) {
                        print!("{:02x} ", byte);
                    }
                    println!();
                }

                // Display key history
                println!("\nKey History:");
                println!("Device 1: {} keys", rotation1.key_history().len());
                for (i, record) in rotation1.key_history().iter().enumerate() {
                    println!("  Key {}: {:?} ({})", 
                        i + 1, 
                        record.status,
                        record.deactivation_reason.as_deref().unwrap_or("N/A")
                    );
                }
            } else {
                println!("\nNo rotation needed");
            }
        } else {
            println!("\nContext establishment failed");
        }

        // Wait before next cycle
        thread::sleep(Duration::from_secs(3));
    }

    // Simulate key invalidation
    println!("\nSimulating key invalidation...");
    rotation1.invalidate_key("Security policy update");
    rotation2.invalidate_key("Security policy update");

    println!("\nFinal Key History:");
    println!("Device 1: {} keys", rotation1.key_history().len());
    for (i, record) in rotation1.key_history().iter().enumerate() {
        println!("  Key {}: {:?} ({})", 
            i + 1, 
            record.status,
            record.deactivation_reason.as_deref().unwrap_or("N/A")
        );
    }

    // Stop collection
    manager1.validator_mut().stop_collection()?;
    manager2.validator_mut().stop_collection()?;

    Ok(())
}
