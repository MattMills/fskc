use crate::Result;
use sha2::{Digest, Sha256};

/// Combines multiple seeds into a single seed using SHA-256.
/// 
/// This function is used internally for combining entropy from multiple sources
/// into a single seed value.
#[allow(dead_code)]
fn combine_seeds(seeds: &[u64]) -> u64 {
    let mut hasher = Sha256::new();
    
    for seed in seeds {
        hasher.update(seed.to_le_bytes());
    }
    
    let result = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&result[..8]);
    
    u64::from_le_bytes(bytes)
}

/// Splits data into chunks of specified size.
/// 
/// Used internally when preparing data for fractal encryption,
/// where data needs to be processed in fixed-size blocks.
#[allow(dead_code)]
fn split_into_chunks(data: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    data.chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

/// Combines multiple data chunks into a single vector.
/// 
/// Used internally as the inverse of `split_into_chunks` when
/// reconstructing data after fractal decryption.
#[allow(dead_code)]
fn combine_chunks(chunks: &[Vec<u8>]) -> Vec<u8> {
    let total_size: usize = chunks.iter().map(|chunk| chunk.len()).sum();
    let mut result = Vec::with_capacity(total_size);
    
    for chunk in chunks {
        result.extend_from_slice(chunk);
    }
    
    result
}

/// Validates that a seed value is non-zero.
/// 
/// Used internally to ensure that seed values meet the minimum
/// security requirements.
#[allow(dead_code)]
fn validate_seed(seed: u64) -> Result<()> {
    if seed == 0 {
        Err(crate::error::FskcError::InvalidSeed)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_seeds() {
        let seeds = vec![1, 2, 3, 4];
        let combined = combine_seeds(&seeds);
        assert_ne!(combined, 0);
        
        // Same inputs should produce same output
        let combined2 = combine_seeds(&seeds);
        assert_eq!(combined, combined2);
    }

    #[test]
    fn test_split_and_combine_chunks() {
        let data = b"Hello, World!".to_vec();
        let chunk_size = 3;
        
        let chunks = split_into_chunks(&data, chunk_size);
        assert!(chunks.iter().all(|chunk| chunk.len() <= chunk_size));
        
        let recombined = combine_chunks(&chunks);
        assert_eq!(data, recombined);
    }

    #[test]
    fn test_validate_seed() {
        assert!(validate_seed(123).is_ok());
        assert!(validate_seed(0).is_err());
    }
}
