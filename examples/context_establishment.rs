use fskc::{
    Result,
    Accelerometer, Barometer,
    pairlet::{CoPresenceValidator, ContextConfig, ContextManager},
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

        // Verify context is valid
        println!("\nContext Validation:");
        println!("Is Valid: {}", 
            if manager1.is_context_valid(&context) { "YES ✓" } else { "NO ✗" });
    } else {
        println!("\nContext Establishment Result: FAILED ✗");
        println!("Could not establish sufficient shared context");
    }

    // Stop collection
    manager1.validator_mut().stop_collection()?;
    manager2.validator_mut().stop_collection()?;

    Ok(())
}
