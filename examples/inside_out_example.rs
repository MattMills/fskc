use fskc::{SystemState, Result};

fn main() -> Result<()> {
    // Create system state with seed
    let mut state = SystemState::new(12345)?;
    
    // Test data
    let data = b"Hello, inside-out crypto!";
    println!("Original: {:?}", String::from_utf8_lossy(data));
    
    // Encrypt
    let encrypted = state.encrypt(data)?;
    println!("Encrypted: {:?}", encrypted);
    
    // Decrypt
    let decrypted = state.decrypt(&encrypted)?;
    println!("Decrypted: {:?}", String::from_utf8_lossy(&decrypted));
    
    // Demonstrate state evolution
    println!("\nDemonstrating state evolution:");
    for i in 0..3 {
        let test_data = format!("Test {}", i).into_bytes();
        let enc = state.encrypt(&test_data)?;
        let dec = state.decrypt(&enc)?;
        println!("Round {}: {:?} -> {:?} -> {:?}",
                i,
                String::from_utf8_lossy(&test_data),
                enc,
                String::from_utf8_lossy(&dec));
    }

    Ok(())
}
