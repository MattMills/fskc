use fskc::{HolographicKeyPackage, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Create a deterministic RNG for the example
    let mut rng = ChaCha20Rng::seed_from_u64(12345);

    // Create a holographic key package
    let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);

    // Add some time steps
    for _ in 0..3 {
        pkg.add_time_step(&mut rng)?;
    }

    // Original message
    let message = b"Hello, Holographic World!".to_vec();
    println!("Original message: {}", String::from_utf8_lossy(&message));

    // Encrypt through time steps
    let mut encrypted = message.clone();
    for i in 0..pkg.num_steps() {
        pkg.apply_forward(&mut encrypted)?;
        println!("After time step {}: {:?}", i, encrypted);
        
        if i + 1 < pkg.num_steps() {
            pkg.advance()?;
        }
    }

    // Decrypt back through time steps
    for i in (0..pkg.num_steps()).rev() {
        pkg.apply_backward(&mut encrypted)?;
        println!("Reversed time step {}: {}", i, String::from_utf8_lossy(&encrypted));
    }

    assert_eq!(message, encrypted);
    println!("\nSuccessfully demonstrated time-sequence based encryption!");

    Ok(())
}
