use crate::{
    Result, FskcError,
    HomomorphicCompute,
};

/// Linear ZKP container with forward/backward proof verification
pub struct ZkpContainer {
    compute: HomomorphicCompute,
    state: Vec<u8>,
    forward_proof: Option<Vec<u8>>,  // Proof for next state
    backward_proof: Option<Vec<u8>>, // Proof for previous state
}

impl ZkpContainer {
    /// Create new ZKP container with initial state
    pub fn new(compute: HomomorphicCompute, initial_state: Vec<u8>) -> Result<Self> {
        Ok(Self {
            compute,
            state: initial_state,
            forward_proof: None,
            backward_proof: None,
        })
    }

    /// Generate proof for transitioning to next state
    pub fn prove_next(&mut self, next_state: &[u8]) -> Result<Vec<u8>> {
        // Generate proof as difference between states
        self.compute.load(0, next_state)?;
        self.compute.load(1, &self.state)?;
        self.compute.compute(crate::Operation::Sub, 0, 1)?;
        
        let proof = self.compute.read(0)?;
        self.forward_proof = Some(proof.clone());
        
        Ok(proof)
    }

    /// Generate proof for transitioning to previous state
    pub fn prove_previous(&mut self, prev_state: &[u8]) -> Result<Vec<u8>> {
        // Generate proof as difference between states
        self.compute.load(0, &self.state)?;
        self.compute.load(1, prev_state)?;
        self.compute.compute(crate::Operation::Sub, 0, 1)?;
        
        let proof = self.compute.read(0)?;
        self.backward_proof = Some(proof.clone());
        
        Ok(proof)
    }

    /// Verify proof for transitioning to next state
    pub fn verify_next(&mut self, next_state: &[u8], proof: &[u8]) -> Result<bool> {
        // Verify next_state = current_state + proof
        self.compute.load(0, &self.state)?;
        self.compute.load(1, proof)?;
        self.compute.compute(crate::Operation::Add, 0, 1)?;
        
        let computed = self.compute.read(0)?;
        Ok(computed == next_state)
    }

    /// Verify proof for transitioning to previous state  
    pub fn verify_previous(&mut self, prev_state: &[u8], proof: &[u8]) -> Result<bool> {
        // Verify current_state = prev_state + proof
        self.compute.load(0, prev_state)?;
        self.compute.load(1, proof)?;
        self.compute.compute(crate::Operation::Add, 0, 1)?;
        
        let computed = self.compute.read(0)?;
        Ok(computed == self.state)
    }

    /// Transition to next state if proof verifies
    pub fn advance(&mut self, next_state: Vec<u8>, proof: &[u8]) -> Result<()> {
        if !self.verify_next(&next_state, proof)? {
            return Err(FskcError::Custom("Invalid next state proof".into()));
        }
        
        self.state = next_state;
        self.forward_proof = None;
        self.backward_proof = None;
        Ok(())
    }

    /// Transition to previous state if proof verifies
    pub fn reverse(&mut self, prev_state: Vec<u8>, proof: &[u8]) -> Result<()> {
        if !self.verify_previous(&prev_state, proof)? {
            return Err(FskcError::Custom("Invalid previous state proof".into())); 
        }
        
        self.state = prev_state;
        self.forward_proof = None;
        self.backward_proof = None;
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> &[u8] {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HolographicKeyPackage;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_zkp_container() -> Result<()> {
        // Initialize compute engine
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let mut key = vec![0u8; 32];
        rng.fill_bytes(&mut key);
        let mut pkg = HolographicKeyPackage::new(key);
        pkg.add_time_step(&mut rng)?;
        let compute = HomomorphicCompute::new(pkg)?;

        // Create container with initial state
        let initial_state = vec![1u8; 32];
        let mut container = ZkpContainer::new(compute, initial_state.clone())?;

        // Generate proof for next state
        let next_state = vec![2u8; 32];
        let forward_proof = container.prove_next(&next_state)?;

        // Verify and advance
        assert!(container.verify_next(&next_state, &forward_proof)?);
        container.advance(next_state.clone(), &forward_proof)?;
        assert_eq!(container.state(), &next_state);

        // Generate proof for previous state
        let backward_proof = container.prove_previous(&initial_state)?;

        // Verify and reverse
        assert!(container.verify_previous(&initial_state, &backward_proof)?);
        container.reverse(initial_state.clone(), &backward_proof)?;
        assert_eq!(container.state(), &initial_state);

        Ok(())
    }
}
