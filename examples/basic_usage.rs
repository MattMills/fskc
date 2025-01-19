use fskc::{FractalNode, RovingSelector, Result};

fn main() -> Result<()> {
    // Example message to encrypt
    let message = b"This is a secret message that will be encrypted using fractal structures!".to_vec();
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

    println!("Generated seed from roving selector: {}", combined_seed);

    // Now use the fractal structure to encrypt the message
    let chunk_size = 16; // Size of data chunks for fractal structure
    let depth = 3; // Depth of the fractal structure
    
    // Generate the fractal structure (encrypts the data)
    let encrypted = FractalNode::generate(message.clone(), combined_seed, depth, chunk_size)?;
    
    println!("\nMessage has been encrypted using fractal structure");
    println!("Fractal depth: {}", encrypted.depth());
    println!("Number of child nodes: {}", encrypted.child_count());

    // Decrypt the message
    let decrypted = encrypted.decrypt()?;
    
    println!("\nDecrypted message: {}", String::from_utf8_lossy(&decrypted));
    
    // Verify the decryption was successful
    assert_eq!(message, decrypted, "Decryption failed!");
    println!("\nVerification successful - decrypted message matches original!");

    Ok(())
}
