use fskc::{
    Result,
    Accelerometer, Barometer,
    pairlet::{
        CoPresenceValidator, ContextConfig, ContextManager,
        KeyGenConfig, KeyGenerator, ExchangeConfig, KeyExchange,
        RotationConfig, KeyRotation, RecoveryConfig, KeyRecovery,
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

    // Configure key recovery
    let recovery_config = RecoveryConfig {
        max_context_age: Duration::from_secs(60),      // Short age for demo
        min_recovery_quality: 0.9,
        confirmation_rounds: 5,                        // Extra rounds for recovery
        max_recovery_attempts: 3,
    };
    let confirmation_rounds = recovery_config.confirmation_rounds;

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
        KeyGenerator::new(keygen_config.clone()),
    );

    // Create key recovery managers
    let mut recovery1 = KeyRecovery::new(
        recovery_config.clone(),
        KeyGenerator::new(keygen_config.clone()),
    );
    let mut recovery2 = KeyRecovery::new(
        recovery_config,
        KeyGenerator::new(keygen_config),
    );

    // Start collecting measurements
    println!("Starting measurement collection...");
    manager1.validator_mut().start_collection()?;
    manager2.validator_mut().start_collection()?;

    // Wait for measurements
    thread::sleep(Duration::from_secs(3));

    // Get measurement windows from device 2
    let device2_windows = manager2.validator().recent_windows(3);

    // Establish initial context
    println!("\nEstablishing initial context...");
    if let Some(context) = manager1.establish_context(&device2_windows)? {
        println!("Context Quality: {:.2}", context.quality);

        // Generate initial keys
        println!("\nGenerating initial keys...");
        rotation1.rotate_key(&context, &mut exchange1)?;
        rotation2.rotate_key(&context, &mut exchange2)?;

        // Display initial keys
        if let Some(key1) = rotation1.active_key() {
            println!("\nDevice 1 Initial Key:");
            println!("Status: {:?}", key1.status);
            println!("Quality: {:.2}", key1.key.quality);
            print!("Material (first 16 bytes): ");
            for byte in key1.key.key.iter().take(16) {
                print!("{:02x} ", byte);
            }
            println!();
        }

        // Simulate key loss on device 1
        println!("\nSimulating key loss on Device 1...");
        rotation1.invalidate_key("Key lost");
        assert!(rotation1.active_key().is_none());

        // Start recovery process on both devices
        println!("\nStarting key recovery...");
        recovery1.start_recovery(&context)?;
        recovery2.start_recovery(&context)?;
        println!("Recovery Status: {:?}", recovery1.status());

        // Generate recovery keys
        println!("\nGenerating recovery keys...");
        let recovery_key1 = recovery1.generate_recovery_key(&context)?;
        let recovery_key2 = recovery2.generate_recovery_key(&context)?;
        println!("Recovery Keys Generated");
        println!("Device 1 Key Quality: {:.2}", recovery_key1.quality);
        println!("Device 2 Key Quality: {:.2}", recovery_key2.quality);

        // Start key exchange for recovery
        println!("\nStarting recovery key exchange...");
        exchange1.start_exchange(&context)?;
        exchange2.start_exchange(&context)?;

        // Verify recovery with both devices
        println!("\nVerifying recovery...");
        recovery1.verify_recovery(&mut exchange1, &mut rotation1)?;
        recovery2.verify_recovery(&mut exchange2, &mut rotation2)?;

        // Display recovered keys
        if let (Some(key1), Some(key2)) = (rotation1.active_key(), rotation2.active_key()) {
            println!("\nDevice 1 Recovered Key:");
            println!("Status: {:?}", key1.status);
            println!("Quality: {:.2}", key1.key.quality);
            print!("Material (first 16 bytes): ");
            for byte in key1.key.key.iter().take(16) {
                print!("{:02x} ", byte);
            }
            println!();

            println!("\nDevice 2 Recovered Key:");
            println!("Status: {:?}", key2.status);
            println!("Quality: {:.2}", key2.key.quality);
            print!("Material (first 16 bytes): ");
            for byte in key2.key.key.iter().take(16) {
                print!("{:02x} ", byte);
            }
            println!();

            // Verify keys match
            println!("\nKey Verification:");
            println!("Keys Match: {}", 
                if key1.key.key == key2.key.key { "YES ✓" } else { "NO ✗" });
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

        // Verify recovery status
        println!("\nRecovery Status: {:?}", recovery1.status());
        println!("Recovery Attempts: {}", recovery1.attempts());
        println!("Recovery Locked Out: {}", recovery1.is_locked_out());
    } else {
        println!("\nContext establishment failed - Cannot proceed with recovery");
    }

    // Stop collection
    manager1.validator_mut().stop_collection()?;
    manager2.validator_mut().stop_collection()?;

    Ok(())
}
