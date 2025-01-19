use fskc::{
    LayeredCrypto, LayerConfig, RovingSelector,
    EntropyBuilder, Result,
};
use rand::{SeedableRng};
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Example message to encrypt
    let message = b"This is a secret message that will be encrypted using layered fractal structures!".to_vec();
    println!("Original message: {}", String::from_utf8_lossy(&message));

    // First, use the roving selector to generate a seed from some shared data
    let shared_data = b"This could be any public data source that both parties have access to".to_vec();
    let mut selector = RovingSelector::new(8, 5, 12345)?;
    
    // Map the shared data into high-dimensional space
    selector.map_data(&shared_data)?;
    
    // Run the selector for a few steps to generate entropy
    let mut combined_seed = 0u64;
    for _ in 0..5 {
        let selected = selector.step()?;
        // Combine the selected bytes into a seed
        for &byte in &selected {
            combined_seed = combined_seed.wrapping_mul(256).wrapping_add(byte as u64);
        }
    }

    println!("Generated initial seed from roving selector: {}", combined_seed);

    // Example 1: Basic layered encryption (FSKC + AES)
    println!("\nExample 1: Basic layered encryption");
    let config = LayerConfig::builder()
        .add_fractal()
        .add_aes()
        .build();

    let mut crypto = LayeredCrypto::with_config(combined_seed, config.clone());
    let encrypted = crypto.encrypt(&message, combined_seed)?;
    let mut crypto = LayeredCrypto::with_config(combined_seed, config);
    let decrypted = crypto.decrypt(&encrypted, combined_seed)?;
    assert_eq!(message, decrypted);
    println!("Basic layered encryption successful!");

    // Example 2: High-performance configuration with multiple RNGs
    println!("\nExample 2: High-performance configuration with multiple RNGs");
    let config = LayerConfig::builder()
        .add_fractal()
        .add_chacha()
        .fractal_depth(2)  // Reduced depth for performance
        .chunk_size(128)   // Larger chunks for performance
        .build();

    // Create entropy from multiple RNGs
    let rng1 = ChaCha20Rng::seed_from_u64(combined_seed);
    let rng2 = ChaCha20Rng::seed_from_u64(combined_seed.wrapping_add(1));
    let entropy = EntropyBuilder::new()
        .add_rng(rng1, "ChaCha20 RNG 1")
        .add_rng(rng2, "ChaCha20 RNG 2")
        .build();

    let mut crypto = LayeredCrypto::with_entropy(entropy, config.clone());
    let encrypted = crypto.encrypt(&message, combined_seed)?;
    let mut crypto = LayeredCrypto::with_config(combined_seed, config);
    let decrypted = crypto.decrypt(&encrypted, combined_seed)?;
    assert_eq!(message, decrypted);
    println!("High-performance encryption successful!");

    // Example 3: Maximum security with physical entropy
    println!("\nExample 3: Maximum security with physical entropy");
    let config = LayerConfig::builder()
        .add_fractal()
        .add_aes()
        .add_chacha()
        .add_aes()
        .add_fractal()
        .enable_zippering()
        .build();

    // Simulate physical entropy sources
    let ligo_data = vec![0x42; 1024];  // In practice, this would be real LIGO data
    let stellar_data = vec![0x17; 1024]; // In practice, this would be real stellar data
    
    let entropy = EntropyBuilder::new()
        .add_rng(ChaCha20Rng::seed_from_u64(combined_seed), "Base RNG")
        .add_ligo_data(ligo_data)
        .add_stellar_parallax(stellar_data)
        .build();

    let mut crypto = LayeredCrypto::with_entropy(entropy, config.clone());
    let encrypted = crypto.encrypt(&message, combined_seed)?;
    let mut crypto = LayeredCrypto::with_config(combined_seed, config);
    let decrypted = crypto.decrypt(&encrypted, combined_seed)?;
    assert_eq!(message, decrypted);
    println!("Maximum security encryption successful!");

    // Example 4: Custom sequence with mixed entropy
    println!("\nExample 4: Custom sequence with mixed entropy");
    let config = LayerConfig::builder()
        .add_chacha()      // Fast initial scrambling
        .add_fractal()     // Geometric complexity
        .add_aes()         // Strong symmetric encryption
        .chunk_size(256)   // Custom chunk size
        .fractal_depth(4)  // Custom fractal depth
        .build();

    // Create entropy from multiple sources
    let custom_physical = vec![0x89; 1024]; // Custom physical measurements
    let entropy = EntropyBuilder::new()
        .add_rng(ChaCha20Rng::seed_from_u64(combined_seed), "Base RNG")
        .add_physical_source(custom_physical, "Custom Physical Source")
        .build();

    let mut crypto = LayeredCrypto::with_entropy(entropy, config.clone());
    let encrypted = crypto.encrypt(&message, combined_seed)?;
    let mut crypto = LayeredCrypto::with_config(combined_seed, config);
    let decrypted = crypto.decrypt(&encrypted, combined_seed)?;
    assert_eq!(message, decrypted);
    println!("Custom sequence encryption successful!");

    // Print entropy source information
    println!("\nEntropy sources used in examples:");
    println!("1. Basic: ChaCha20 RNG");
    println!("2. High-performance: Multiple ChaCha20 RNGs");
    println!("3. Maximum security: ChaCha20 RNG + LIGO data + Stellar parallax");
    println!("4. Custom: ChaCha20 RNG + Custom physical measurements");

    Ok(())
}
