use fskc::{
    Result,
    Accelerometer, Barometer,
    pairlet::{
        CoPresenceValidator, ContextConfig, ContextManager,
        KeyGenConfig, KeyGenerator, ExchangeConfig, KeyExchange,
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

        // Create key exchanges for both devices
        let mut exchange1 = KeyExchange::new(
            exchange_config.clone(),
            KeyGenerator::new(keygen_config.clone()),
        );
        let mut exchange2 = KeyExchange::new(
            exchange_config,
            KeyGenerator::new(keygen_config),
        );

        // Start key exchange
        println!("\nStarting key exchange...");
        exchange1.start_exchange(&context)?;
        exchange2.start_exchange(&context)?;

        // Perform confirmation rounds
        println!("\nPerforming key confirmation...");
        for round in 0..3 {
            println!("\nConfirmation Round {}", round + 1);
            
            // Generate confirmation hashes
            let hash1 = exchange1.generate_confirmation(round)?;
            let hash2 = exchange2.generate_confirmation(round)?;

            // Verify hashes
            let valid1 = exchange1.verify_confirmation(round, &hash2)?;
            let valid2 = exchange2.verify_confirmation(round, &hash1)?;

            println!("Device 1 Verification: {}", if valid1 { "VALID ✓" } else { "FAILED ✗" });
            println!("Device 2 Verification: {}", if valid2 { "VALID ✓" } else { "FAILED ✗" });

            if !valid1 || !valid2 {
                println!("\nKey Exchange Failed - Hash Mismatch");
                return Ok(());
            }
        }

        // Display final results
        if exchange1.status() == exchange2.status() {
            println!("\nKey Exchange Complete!");
            
            // Get final keys
            let key1 = exchange1.key().unwrap();
            let key2 = exchange2.key().unwrap();

            println!("\nDevice 1 Key:");
            println!("Length: {} bytes", key1.key.len());
            println!("Quality: {:.2}", key1.quality);
            print!("Material (first 16 bytes): ");
            for byte in key1.key.iter().take(16) {
                print!("{:02x} ", byte);
            }

            println!("\n\nDevice 2 Key:");
            println!("Length: {} bytes", key2.key.len());
            println!("Quality: {:.2}", key2.quality);
            print!("Material (first 16 bytes): ");
            for byte in key2.key.iter().take(16) {
                print!("{:02x} ", byte);
            }
            println!();

            // Verify keys match
            println!("\nKey Verification:");
            println!("Keys Match: {}", 
                if key1.key == key2.key { "YES ✓" } else { "NO ✗" });
        } else {
            println!("\nKey Exchange Failed - Status Mismatch");
        }
    } else {
        println!("\nContext Establishment Failed - Cannot Exchange Keys");
    }

    // Stop collection
    manager1.validator_mut().stop_collection()?;
    manager2.validator_mut().stop_collection()?;

    Ok(())
}
