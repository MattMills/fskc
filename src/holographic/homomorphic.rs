use crate::Result;

/// A simplified key package for homomorphic operations
pub struct HomomorphicKeyPackage {
    key: Vec<u8>,
}

impl HomomorphicKeyPackage {
    /// Creates a new homomorphic key package
    pub fn new(key: Vec<u8>) -> Self {
        Self { key }
    }

    /// Encrypts data using XOR (which preserves homomorphic properties)
    pub fn encrypt(&self, data: &mut [u8]) -> Result<()> {
        for (chunk, &key_byte) in data.iter_mut().zip(self.key.iter().cycle()) {
            *chunk ^= key_byte;
        }
        Ok(())
    }

    /// Decrypts data (same as encrypt since XOR is its own inverse)
    pub fn decrypt(&self, data: &mut [u8]) -> Result<()> {
        self.encrypt(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homomorphic_xor() {
        let key = vec![0x42; 32];
        let pkg = HomomorphicKeyPackage::new(key);

        // Original values
        let num1 = 25u8;
        let num2 = 17u8;

        // Encrypt values
        let mut encrypted1 = vec![num1];
        let mut encrypted2 = vec![num2];
        pkg.encrypt(&mut encrypted1).unwrap();
        pkg.encrypt(&mut encrypted2).unwrap();

        // XOR the encrypted values
        let mut encrypted_xor = vec![encrypted1[0] ^ encrypted2[0]];

        // Decrypt the result
        pkg.decrypt(&mut encrypted_xor).unwrap();

        // Verify homomorphic XOR property
        assert_eq!(encrypted_xor[0], num1 ^ num2);
    }

    #[test]
    fn test_homomorphic_and() {
        let key = vec![0x42; 32];
        let pkg = HomomorphicKeyPackage::new(key);

        // Original value and mask
        let num = 25u8;
        let mask = 3u8;

        // Encrypt value
        let mut encrypted = vec![num];
        pkg.encrypt(&mut encrypted).unwrap();

        // AND with mask (directly on encrypted data)
        let mut encrypted_and = vec![encrypted[0] & mask];

        // Decrypt the result
        pkg.decrypt(&mut encrypted_and).unwrap();

        // Verify homomorphic AND property
        assert_eq!(encrypted_and[0], num & mask);
    }
}
