use crate::{crypto, error::FskcError, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use std::sync::Arc;

/// Represents a node in the fractal encryption structure
#[derive(Debug, Clone)]
pub struct FractalNode {
    /// The encrypted data block at this node
    value: Vec<u8>,
    /// The seed used for RNG at this node
    seed: u64,
    /// Child nodes in the fractal structure
    children: Vec<Arc<FractalNode>>,
    /// Depth level in the fractal structure
    depth: usize,
}

impl FractalNode {
    /// Creates a new FractalNode with the given parameters
    pub fn new(value: Vec<u8>, seed: u64, depth: usize) -> Self {
        Self {
            value,
            seed,
            children: Vec::new(),
            depth,
        }
    }

    /// Generates a fractal structure from the input data
    pub fn generate(
        data: Vec<u8>,
        initial_seed: u64,
        max_depth: usize,
        chunk_size: usize,
    ) -> Result<Arc<Self>> {
        // Validate parameters
        if max_depth == 0 {
            return Err(FskcError::InvalidDepth(0));
        }

        if data.is_empty() {
            return Err(FskcError::InvalidDataSize(0));
        }

        if chunk_size == 0 {
            return Err(FskcError::InvalidDataSize(chunk_size));
        }

        // Initialize RNG with the seed
        let mut rng = ChaCha20Rng::seed_from_u64(initial_seed);
        
        // XOR the data with RNG output
        let encrypted_data = crypto::xor_with_rng(&data, &mut rng)?;

        let mut node = Self::new(encrypted_data, initial_seed, max_depth);

        // Only create child nodes if we haven't reached max depth
        if max_depth > 1 && data.len() > chunk_size {
            // Split data into chunks
            let chunks: Vec<Vec<u8>> = data
                .chunks(chunk_size)
                .map(|chunk| chunk.to_vec())
                .collect();

            // Create child nodes recursively
            for (i, chunk) in chunks.into_iter().enumerate() {
                // Generate a new seed for each child
                let child_seed = initial_seed.wrapping_add(i as u64);
                
                let child = Self::generate(
                    chunk,
                    child_seed,
                    max_depth - 1,
                    chunk_size,
                )?;
                
                node.children.push(child);
            }
        }

        Ok(Arc::new(node))
    }

    /// Decrypts the fractal structure and returns the original data
    pub fn decrypt(&self) -> Result<Vec<u8>> {
        // Initialize RNG with the node's seed
        let mut rng = ChaCha20Rng::seed_from_u64(self.seed);
        
        // Decrypt this node's value
        let mut decrypted = crypto::xor_with_rng(&self.value, &mut rng)?;

        // If this node has children, decrypt them and combine the results
        if !self.children.is_empty() {
            let mut child_data = Vec::new();
            
            for child in &self.children {
                let child_decrypted = child.decrypt()?;
                child_data.extend(child_decrypted);
            }
            
            // Replace the decrypted data with the combined child data
            decrypted = child_data;
        }

        Ok(decrypted)
    }

    /// Returns the depth of this node in the fractal structure
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the number of child nodes
    pub fn child_count(&self) -> usize {
        self.children.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fractal_encryption_decryption() {
        let data = b"Hello, Fractal World!".to_vec();
        let seed = 12345;
        let depth = 3;
        let chunk_size = 4;

        // Generate fractal structure
        let node = FractalNode::generate(data.clone(), seed, depth, chunk_size)
            .expect("Failed to generate fractal");

        // Decrypt the data
        let decrypted = node.decrypt().expect("Failed to decrypt");

        // Verify the decryption
        assert_eq!(data, decrypted, "Decrypted data doesn't match original");
    }

    #[test]
    fn test_invalid_depth() {
        let data = vec![1, 2, 3, 4];
        let result = FractalNode::generate(data, 12345, 0, 2);
        assert!(matches!(result, Err(FskcError::InvalidDepth(0))));
    }

    #[test]
    fn test_empty_data() {
        let data = vec![];
        let result = FractalNode::generate(data, 12345, 3, 2);
        assert!(matches!(result, Err(FskcError::InvalidDataSize(0))));
    }
}
