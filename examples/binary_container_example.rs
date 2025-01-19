use fskc::{BinaryContainer, Result};

use rand::{SeedableRng, RngCore};
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Create original container with 3 layers
    let rng = ChaCha20Rng::seed_from_u64(12345);
    let mut original = BinaryContainer::new(rng, 3)?;
    
    // Generate one-time pad
    let mut pad = vec![0u8; 32];
    let mut pad_rng = ChaCha20Rng::seed_from_u64(54321);
    pad_rng.fill_bytes(&mut pad);
    
    // Clone container with pad
    let mut clone = original.clone_with_pad(&pad)?;
    
    println!("Initial state:");
    println!("Original: {:?}", original.state());
    println!("Clone: {:?}", clone.state());
    let result = original.interact(&mut clone)?;
    println!("Verification: {:?}\n", result);
    
    // Demonstrate synchronized evolution
    println!("Synchronized evolution:");
    for i in 0..3 {
        original.iterate()?;
        clone.iterate()?;
        println!("Iteration {}:", i + 1);
        println!("Original: {:?}", original.state());
        println!("Clone: {:?}", clone.state());
        let result = original.interact(&mut clone)?;
        println!("Verification: {:?}\n", result);
    }
    
    // Demonstrate relationship breaking with independent evolution
    println!("Independent evolution (breaks outer layer):");
    let mut independent = clone;
    
    original.iterate()?;
    independent.iterate()?;
    independent.iterate()?; // Extra iteration breaks synchronization
    
    println!("Original (iteration {}): {:?}", 
             original.iteration(), original.state());
    println!("Independent (iteration {}): {:?}", 
             independent.iteration(), independent.state());
    let result = original.interact(&mut independent)?;
    println!("Verification: {:?}", result);

    Ok(())
}
