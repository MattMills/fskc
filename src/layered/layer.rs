use std::sync::Arc;
use crate::{Result, FskcError, FractalNode};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use rand::RngCore;

/// Represents different types of encryption layers
#[derive(Clone)]
pub enum Layer {
    Fractal(FractalLayer),
    Symmetric(SymmetricLayer),
}

/// Represents a fractal-based encryption layer
#[derive(Clone)]
pub struct FractalLayer {
    // Additional configuration could be added here
}

impl FractalLayer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn generate(
        &self,
        data: Vec<u8>,
        seed: u64,
        depth: usize,
        chunk_size: usize,
    ) -> Result<Arc<FractalNode>> {
        FractalNode::generate(data, seed, depth, chunk_size)
    }
}

/// Available symmetric encryption algorithms
#[derive(Clone)]
pub enum SymmetricLayer {
    Aes,
    ChaCha,
}

impl SymmetricLayer {
    pub fn encrypt(&self, rng: &mut impl RngCore, data: &[u8]) -> Result<Vec<u8>> {
        match self {
            SymmetricLayer::Aes => {
                // Generate key and nonce
                let mut key = [0u8; 32];
                rng.fill_bytes(&mut key);
                let mut nonce_bytes = [0u8; 12];
                rng.fill_bytes(&mut nonce_bytes);

                // Create cipher and encrypt
                let cipher = Aes256Gcm::new_from_slice(&key)
                    .map_err(|e| FskcError::EncryptionError(e.to_string()))?;
                let nonce = Nonce::from_slice(&nonce_bytes);

                // Prepend key and nonce to encrypted data
                let mut result = Vec::with_capacity(44 + data.len());
                result.extend_from_slice(&key);
                result.extend_from_slice(&nonce_bytes);
                result.extend(
                    cipher
                        .encrypt(nonce, data)
                        .map_err(|e| FskcError::EncryptionError(e.to_string()))?
                );
                Ok(result)
            }
            SymmetricLayer::ChaCha => {
                // Generate key and nonce
                let mut key = [0u8; 32];
                rng.fill_bytes(&mut key);
                let mut nonce_bytes = [0u8; 12];
                rng.fill_bytes(&mut nonce_bytes);

                // Create cipher
                let mut cipher = ChaCha20::new(
                    key.as_slice().into(),
                    nonce_bytes.as_slice().into(),
                );

                // Encrypt data
                let mut encrypted = data.to_vec();
                cipher.apply_keystream(&mut encrypted);

                // Prepend key and nonce to encrypted data
                let mut result = Vec::with_capacity(44 + encrypted.len());
                result.extend_from_slice(&key);
                result.extend_from_slice(&nonce_bytes);
                result.extend(encrypted);
                Ok(result)
            }
        }
    }

    pub fn decrypt(&self, _rng: &mut impl RngCore, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 44 {
            return Err(FskcError::EncryptionError("Invalid data length".into()));
        }

        // Split data into key, nonce, and ciphertext
        let (key, rest) = data.split_at(32);
        let (nonce_bytes, ciphertext) = rest.split_at(12);

        match self {
            SymmetricLayer::Aes => {
                // Create cipher and decrypt
                let cipher = Aes256Gcm::new_from_slice(key)
                    .map_err(|e| FskcError::EncryptionError(e.to_string()))?;
                let nonce = Nonce::from_slice(nonce_bytes);

                cipher
                    .decrypt(nonce, ciphertext)
                    .map_err(|e| FskcError::EncryptionError(e.to_string()))
            }
            SymmetricLayer::ChaCha => {
                // Create cipher
                let mut cipher = ChaCha20::new(
                    key.into(),
                    nonce_bytes.into(),
                );

                // Decrypt data
                let mut decrypted = ciphertext.to_vec();
                cipher.apply_keystream(&mut decrypted);
                Ok(decrypted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_symmetric_layers() {
        let data = b"Test symmetric encryption".to_vec();
        let mut rng = ChaCha20Rng::seed_from_u64(12345);

        for layer in [SymmetricLayer::Aes, SymmetricLayer::ChaCha] {
            let mut rng_clone = rng.clone();
            let encrypted = layer.encrypt(&mut rng, &data).unwrap();
            let decrypted = layer.decrypt(&mut rng_clone, &encrypted).unwrap();
            assert_eq!(data, decrypted);
        }
    }

    #[test]
    fn test_fractal_layer() {
        let data = b"Test fractal layer".to_vec();
        let layer = FractalLayer::new();
        
        let node = layer.generate(data.clone(), 12345, 3, 64).unwrap();
        let decrypted = node.decrypt().unwrap();
        
        assert_eq!(data, decrypted);
    }
}
