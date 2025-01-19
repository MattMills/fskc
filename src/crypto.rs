use crate::Result;
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Performs XOR operation between data and RNG output
pub fn xor_with_rng(data: &[u8], rng: &mut impl RngCore) -> Result<Vec<u8>> {
    let mut result = Vec::with_capacity(data.len());
    let mut rng_bytes = vec![0u8; data.len()];
    
    // Generate random bytes
    rng.fill_bytes(&mut rng_bytes);
    
    // XOR the data with the random bytes
    for (d, r) in data.iter().zip(rng_bytes.iter()) {
        result.push(d ^ r);
    }
    
    Ok(result)
}

/// Generates a sequence of random bytes using ChaCha20.
/// 
/// This function is used internally by the library for generating random sequences
/// from seed values. It uses the ChaCha20 algorithm for cryptographically secure
/// random number generation.
/// 
/// # Arguments
/// * `seed` - The seed value for the RNG
/// * `length` - The number of random bytes to generate
/// 
/// # Returns
/// A vector containing the generated random bytes
#[allow(dead_code)]
fn generate_random_bytes(seed: u64, length: usize) -> Result<Vec<u8>> {
    let mut rng = ChaCha20Rng::seed_from_u64(seed);
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_roundtrip() {
        let data = b"Test data for XOR".to_vec();
        let seed = 12345;
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        
        // Encrypt
        let encrypted = xor_with_rng(&data, &mut rng).unwrap();
        
        // Create new RNG with same seed for decryption
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        
        // Decrypt
        let decrypted = xor_with_rng(&encrypted, &mut rng).unwrap();
        
        assert_eq!(data, decrypted);
    }

    #[test]
    fn test_random_bytes_deterministic() {
        let seed = 12345;
        let length = 16;
        
        // Generate two sequences with same seed
        let bytes1 = generate_random_bytes(seed, length).unwrap();
        let bytes2 = generate_random_bytes(seed, length).unwrap();
        
        // They should be identical
        assert_eq!(bytes1, bytes2);
        
        // Generate sequence with different seed
        let bytes3 = generate_random_bytes(seed + 1, length).unwrap();
        
        // Should be different
        assert_ne!(bytes1, bytes3);
    }
}
