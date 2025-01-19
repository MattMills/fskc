use fskc::{FractalNode, RovingSelector, Result};
use proptest::prelude::*;
use test_case::test_case;

// Helper function to run an encryption-decryption cycle
fn encrypt_decrypt_cycle(
    data: Vec<u8>,
    shared_data: Vec<u8>,
    dimension: usize,
    particles: usize,
    depth: usize,
    chunk_size: usize,
) -> Result<()> {
    // Generate seed using roving selector
    let mut selector = RovingSelector::new(dimension, particles, 12345)?;
    selector.map_data(&shared_data)?;
    
    let mut combined_seed = 0u64;
    for _ in 0..5 {
        let selected = selector.step()?;
        for &byte in &selected {
            combined_seed = combined_seed.wrapping_mul(256).wrapping_add(byte as u64);
        }
    }

    // Encrypt data
    let encrypted = FractalNode::generate(data.clone(), combined_seed, depth, chunk_size)?;
    
    // Decrypt data
    let decrypted = encrypted.decrypt()?;
    
    // Verify
    assert_eq!(data, decrypted, "Decrypted data doesn't match original");
    Ok(())
}

#[test]
fn test_empty_shared_data() {
    let data = b"Test message".to_vec();
    let shared_data = vec![];
    
    let result = encrypt_decrypt_cycle(
        data,
        shared_data,
        8,
        5,
        3,
        16,
    );
    
    assert!(result.is_err());
}

#[test_case(0, 5, 3, 16 ; "invalid dimension")]
#[test_case(8, 0, 3, 16 ; "zero particles")]
#[test_case(8, 5, 0, 16 ; "zero depth")]
#[test_case(8, 5, 3, 0  ; "zero chunk size")]
fn test_invalid_parameters(dimension: usize, particles: usize, depth: usize, chunk_size: usize) {
    let data = b"Test message".to_vec();
    let shared_data = b"Shared data".to_vec();
    
    let result = encrypt_decrypt_cycle(
        data,
        shared_data,
        dimension,
        particles,
        depth,
        chunk_size,
    );
    
    assert!(result.is_err());
}

#[test_case(4  ; "small chunks")]
#[test_case(8  ; "medium chunks")]
#[test_case(16 ; "large chunks")]
#[test_case(32 ; "extra large chunks")]
fn test_various_chunk_sizes(chunk_size: usize) {
    let data = b"This is a test message for various chunk sizes".to_vec();
    let shared_data = b"Shared data source".to_vec();
    
    let result = encrypt_decrypt_cycle(
        data,
        shared_data,
        8,
        5,
        3,
        chunk_size,
    );
    assert!(result.is_ok(), "Failed with chunk size {}", chunk_size);
}

#[test_case(1 ; "depth one")]
#[test_case(2 ; "depth two")]
#[test_case(3 ; "depth three")]
#[test_case(4 ; "depth four")]
#[test_case(5 ; "depth five")]
fn test_various_depths(depth: usize) {
    let data = b"Test message for various depths".to_vec();
    let shared_data = b"Shared data source".to_vec();
    
    let result = encrypt_decrypt_cycle(
        data,
        shared_data,
        8,
        5,
        depth,
        16,
    );
    assert!(result.is_ok(), "Failed with depth {}", depth);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    // Test with random data of various sizes
    #[test]
    fn test_random_data(
        ref data in prop::collection::vec(any::<u8>(), 1..1000),
        ref shared in prop::collection::vec(any::<u8>(), 1..100),
        dimension in 4..16usize,
        particles in 3..10usize,
        depth in 1..5usize,
        chunk_size in prop::sample::select(vec![4, 8, 16, 32]),
    ) {
        let result = encrypt_decrypt_cycle(
            data.clone(),
            shared.clone(),
            dimension,
            particles,
            depth,
            chunk_size,
        );
        prop_assert!(result.is_ok());
    }
    
    // Test with specific patterns in data
    #[test]
    fn test_pattern_data(
        ref pattern in "[a-zA-Z0-9]{1,100}",
        repeat in 1..10usize,
    ) {
        let data = pattern.repeat(repeat).into_bytes();
        let shared_data = b"Standard shared data".to_vec();
        
        let result = encrypt_decrypt_cycle(
            data,
            shared_data,
            8,
            5,
            3,
            16,
        );
        prop_assert!(result.is_ok());
    }
}

// Test error conditions
mod error_tests {
    use super::*;

    #[test]
    fn test_zero_depth() {
        let data = vec![1, 2, 3, 4];
        let result = FractalNode::generate(data, 12345, 0, 2);
        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let result = FractalNode::generate(data, 12345, 3, 2);
        assert!(matches!(result, Err(_)));
    }

    #[test]
    fn test_invalid_chunk_size() {
        let data = vec![1, 2, 3, 4];
        let result = FractalNode::generate(data, 12345, 3, 0);
        assert!(matches!(result, Err(_)));
    }
}
