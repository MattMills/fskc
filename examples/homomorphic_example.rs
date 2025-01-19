use fskc::{HolographicKeyPackage, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Create a deterministic RNG for the example
    let mut rng = ChaCha20Rng::seed_from_u64(12345);

    // Create a holographic key package
    let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);

    // Add some time steps
    for _ in 0..2 {
        pkg.add_time_step(&mut rng)?;
    }

    // Original numbers to operate on
    let num1 = vec![25u8];
    let num2 = vec![17u8];
    println!("Original numbers: {} and {}", num1[0], num2[0]);

    // Encrypt both numbers
    let mut encrypted1 = num1.clone();
    let mut encrypted2 = num2.clone();
    pkg.apply_forward(&mut encrypted1)?;
    pkg.apply_forward(&mut encrypted2)?;
    println!("Encrypted values: {:?} and {:?}", encrypted1, encrypted2);

    // Perform homomorphic XOR operation
    let xor_result = pkg.homomorphic_operation(&encrypted1, &encrypted2)?;
    println!("Homomorphic XOR result: {:?}", xor_result);

    // Decrypt the result
    let mut decrypted = xor_result;
    pkg.apply_backward(&mut decrypted)?;
    println!("Decrypted result: {}", decrypted[0]);

    // Verify the result matches direct XOR
    let expected = num1[0] ^ num2[0];
    assert_eq!(decrypted[0], expected);
    println!("\nVerified: {} XOR {} = {}", num1[0], num2[0], expected);

    // Demonstrate time sequence property
    println!("\nAdvancing to next time step...");
    pkg.advance()?;

    // Encrypt and operate at new time step
    let mut new_encrypted1 = num1.clone();
    let mut new_encrypted2 = num2.clone();
    pkg.apply_forward(&mut new_encrypted1)?;
    pkg.apply_forward(&mut new_encrypted2)?;

    let new_xor = pkg.homomorphic_operation(&new_encrypted1, &new_encrypted2)?;
    println!("New time step encrypted values: {:?} and {:?}", new_encrypted1, new_encrypted2);
    println!("New time step XOR result: {:?}", new_xor);

    let mut new_decrypted = new_xor;
    pkg.apply_backward(&mut new_decrypted)?;
    assert_eq!(new_decrypted[0], expected);
    println!("Verified homomorphic property preserves through time steps!");

    // Demonstrate associativity
    let num3 = vec![42u8];
    println!("\nTesting associativity with third number: {}", num3[0]);

    let mut encrypted3 = num3.clone();
    pkg.apply_forward(&mut encrypted3)?;

    // (a XOR b) XOR c
    let left_xor = pkg.homomorphic_operation(&new_encrypted1, &new_encrypted2)?;
    let left_final = pkg.homomorphic_operation(&left_xor, &encrypted3)?;
    let mut left_decrypted = left_final;
    pkg.apply_backward(&mut left_decrypted)?;

    // a XOR (b XOR c)
    let right_xor = pkg.homomorphic_operation(&new_encrypted2, &encrypted3)?;
    let right_final = pkg.homomorphic_operation(&new_encrypted1, &right_xor)?;
    let mut right_decrypted = right_final;
    pkg.apply_backward(&mut right_decrypted)?;

    assert_eq!(left_decrypted, right_decrypted);
    println!("Verified associativity: (a XOR b) XOR c = a XOR (b XOR c) = {}", left_decrypted[0]);

    Ok(())
}
