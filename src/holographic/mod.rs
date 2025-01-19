pub mod compute;

use crate::{Result, FskcError};
use rand::{RngCore, CryptoRng};

/// Represents a time step in the homomorphic sequence
#[derive(Debug, Clone)]
struct TimeStep {
    key: Vec<u8>,
    operation: fn(u8, u8) -> u8,
    inverse: fn(u8, u8) -> u8,
}

/// Root key package for homomorphic operations
#[derive(Clone, Debug)]
pub struct HolographicKeyPackage {
    // Sequence of time steps for homomorphic operations
    time_steps: Vec<TimeStep>,
    // Current position in the sequence
    current_step: usize,
    // Root key for initialization
    root_key: Vec<u8>,
}

/// Derived key package for enclave operations
#[derive(Clone, Debug)]
pub struct DerivedKeyPackage {
    // Sequence of time steps derived from root
    time_steps: Vec<TimeStep>,
    // Current position in the sequence
    current_step: usize,
    // Key derived from root key
    derived_key: Vec<u8>,
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
    /// Creates a new root key package
    pub fn new(root_key: Vec<u8>) -> Self {
        Self {
            time_steps: vec![TimeStep::new(root_key.clone())],
            current_step: 0,
            root_key,
        }
    }

    /// Derive a new key package for enclave operations
    pub fn derive_enclave_key(&self) -> Result<DerivedKeyPackage> {
        // Derive a new key using the root key as input
        let mut derived_key = self.root_key.clone();
        // XOR with a constant to create a different but deterministic key
        for byte in derived_key.iter_mut() {
            *byte ^= 0x55; // Simple derivation for example
        }

        Ok(DerivedKeyPackage {
            time_steps: vec![TimeStep::new(derived_key.clone())],
            current_step: 0,
            derived_key,
        })
    }

    /// Adds a new time step to the sequence
    pub fn add_time_step<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<()> {
        let mut key = vec![0u8; self.root_key.len()];
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

impl DerivedKeyPackage {
    /// Adds a new time step to the sequence
    pub fn add_time_step<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> Result<()> {
        let mut key = vec![0u8; self.derived_key.len()];
        rng.fill_bytes(&mut key);
        // XOR with derived key to maintain separation from root
        for (k, &d) in key.iter_mut().zip(self.derived_key.iter()) {
            *k ^= d;
        }
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

    /// Advances to the next time step
    pub fn advance(&mut self) -> Result<()> {
        if self.current_step + 1 < self.time_steps.len() {
            self.current_step += 1;
            Ok(())
        } else {
            Err(FskcError::EncryptionError("No more time steps available".into()))
        }
    }

    /// Performs a homomorphic operation in the derived key space
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
    fn test_key_derivation() -> Result<()> {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let root_pkg = HolographicKeyPackage::new(vec![0x42; 32]);
        let derived_pkg = root_pkg.derive_enclave_key()?;

        // Test that derived key is different but deterministic
        assert_ne!(root_pkg.root_key, derived_pkg.derived_key);
        let derived_pkg2 = root_pkg.derive_enclave_key()?;
        assert_eq!(derived_pkg.derived_key, derived_pkg2.derived_key);

        Ok(())
    }

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

        Ok(())
    }

    #[test]
    fn test_derived_operations() -> Result<()> {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let root_pkg = HolographicKeyPackage::new(vec![0x42; 32]);
        let mut derived_pkg = root_pkg.derive_enclave_key()?;
        derived_pkg.add_time_step(&mut rng)?;

        // Test values
        let num1 = vec![25u8];
        let num2 = vec![17u8];
        let expected_xor = vec![num1[0] ^ num2[0]];

        // Test homomorphic XOR in derived space
        let mut encrypted1 = num1.clone();
        let mut encrypted2 = num2.clone();

        derived_pkg.apply_forward(&mut encrypted1)?;
        derived_pkg.apply_forward(&mut encrypted2)?;

        // XOR in derived homomorphic domain
        let xor_result = derived_pkg.homomorphic_operation(&encrypted1, &encrypted2)?;
        
        // Decrypt the XOR result
        let mut decrypted_xor = xor_result;
        derived_pkg.apply_backward(&mut decrypted_xor)?;
        
        // Should match XOR of original values
        assert_eq!(decrypted_xor, expected_xor, "Derived homomorphic XOR failed");

        Ok(())
    }
}
