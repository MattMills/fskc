pub mod compute;

use crate::{Result, FskcError};
use rand::{RngCore, CryptoRng};

/// Represents a time step in the homomorphic sequence
#[derive(Clone)]
struct TimeStep {
    key: Vec<u8>,
    operation: fn(u8, u8) -> u8,
    inverse: fn(u8, u8) -> u8,
}

/// Represents a holographic key package with time-sequence based homomorphic operations
#[derive(Clone)]
pub struct HolographicKeyPackage {
    // Sequence of time steps for homomorphic operations
    time_steps: Vec<TimeStep>,
    // Current position in the sequence
    current_step: usize,
    // Base key for initialization
    base_key: Vec<u8>,
}

impl TimeStep {
    fn new(key: Vec<u8>) -> Self {
        Self {
            key,
            operation: |a, b| a ^ b,  // XOR for forward operation
            inverse: |a, b| a ^ b,    // XOR is its own inverse
        }
    }

    fn apply(&self, data: &mut [u8], forward: bool) {
        let op = if forward { self.operation } else { self.inverse };
        for (chunk, &key_byte) in data.iter_mut().zip(self.key.iter().cycle()) {
            *chunk = op(*chunk, key_byte);
        }
    }
}

impl HolographicKeyPackage {
    /// Creates a new holographic key package
    pub fn new(base_key: Vec<u8>) -> Self {
        Self {
            time_steps: vec![TimeStep::new(base_key.clone())],
            current_step: 0,
            base_key,
        }
    }

    /// Adds a new time step to the sequence
    pub fn add_time_step<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<()> {
        let mut key = vec![0u8; self.base_key.len()];
        rng.fill_bytes(&mut key);
        self.time_steps.push(TimeStep::new(key));
        Ok(())
    }

    /// Applies the sequence up to the current time step
    pub fn apply_forward(&self, data: &mut [u8]) -> Result<()> {
        for step in self.time_steps[..=self.current_step].iter() {
            step.apply(data, true);
        }
        Ok(())
    }

    /// Reverses the sequence from the current time step
    pub fn apply_backward(&self, data: &mut [u8]) -> Result<()> {
        for step in self.time_steps[..=self.current_step].iter().rev() {
            step.apply(data, false);
        }
        Ok(())
    }

    /// Returns the total number of time steps
    pub fn num_steps(&self) -> usize {
        self.time_steps.len()
    }

    /// Returns the current time step index
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Advances to the next time step
    pub fn advance(&mut self) -> Result<()> {
        if self.current_step + 1 < self.time_steps.len() {
            self.current_step += 1;
            Ok(())
        } else {
            Err(FskcError::EncryptionError("No more time steps available".into()))
        }
    }

    /// Performs a homomorphic XOR operation on encrypted data
    pub fn homomorphic_operation(&self, a: &[u8], b: &[u8]) -> Result<Vec<u8>> {
        if a.len() != b.len() {
            return Err(FskcError::EncryptionError("Input lengths must match".into()));
        }

        // Create temporary copies for decryption
        let mut temp_a = a.to_vec();
        let mut temp_b = b.to_vec();

        // Decrypt both inputs to get back to homomorphic domain
        for step in self.time_steps[..=self.current_step].iter().rev() {
            step.apply(&mut temp_a, false);
            step.apply(&mut temp_b, false);
        }

        // Perform XOR operation in homomorphic domain
        let mut result = Vec::with_capacity(a.len());
        for (&a_byte, &b_byte) in temp_a.iter().zip(temp_b.iter()) {
            result.push(a_byte ^ b_byte);
        }

        // Re-encrypt the result
        for step in self.time_steps[..=self.current_step].iter() {
            step.apply(&mut result, true);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_time_sequence_encryption() -> Result<()> {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);

        // Add some time steps
        for _ in 0..3 {
            pkg.add_time_step(&mut rng)?;
        }

        // Test encryption/decryption at each time step
        let original = b"Test message".to_vec();
        let mut encrypted = original.clone();

        // Test each time step
        while pkg.current_step() + 1 < pkg.num_steps() {
            pkg.apply_forward(&mut encrypted)?;
            assert_ne!(original, encrypted);
            pkg.apply_backward(&mut encrypted)?;
            assert_eq!(original, encrypted);
            pkg.advance()?;
        }

        Ok(())
    }

    #[test]
    fn test_homomorphic_operations() -> Result<()> {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);
        pkg.add_time_step(&mut rng)?;

        // Test values
        let num1 = vec![25u8];
        let num2 = vec![17u8];
        let expected_xor = vec![num1[0] ^ num2[0]];

        // Test homomorphic XOR at current time step
        let mut encrypted1 = num1.clone();
        let mut encrypted2 = num2.clone();

        pkg.apply_forward(&mut encrypted1)?;
        pkg.apply_forward(&mut encrypted2)?;

        // XOR in homomorphic domain (after encryption)
        let xor_result = pkg.homomorphic_operation(&encrypted1, &encrypted2)?;
        
        // Decrypt the XOR result
        let mut decrypted_xor = xor_result;
        pkg.apply_backward(&mut decrypted_xor)?;
        
        // Should match XOR of original values
        assert_eq!(decrypted_xor, expected_xor, "Homomorphic XOR failed");

        // Test associativity: (a XOR b) XOR c = a XOR (b XOR c)
        let num3 = vec![42u8];
        let mut encrypted3 = num3.clone();
        pkg.apply_forward(&mut encrypted3)?;

        // Left side: (a XOR b) XOR c
        let left_xor = pkg.homomorphic_operation(&encrypted1, &encrypted2)?;
        let left_final = pkg.homomorphic_operation(&left_xor, &encrypted3)?;

        // Right side: a XOR (b XOR c)
        let right_xor = pkg.homomorphic_operation(&encrypted2, &encrypted3)?;
        let right_final = pkg.homomorphic_operation(&encrypted1, &right_xor)?;

        assert_eq!(left_final, right_final, "Homomorphic associativity failed");

        Ok(())
    }

    #[test]
    fn test_time_step_generation() -> Result<()> {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);

        // Add multiple time steps
        for _ in 0..4 {
            pkg.add_time_step(&mut rng)?;
        }

        // Verify number of time steps (initial + added)
        assert_eq!(pkg.num_steps(), 5);
        Ok(())
    }
}
