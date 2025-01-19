use crate::{
    Result,
    HolographicKeyPackage, HomomorphicCompute, Operation,
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Represents a layered binary container with nested states
pub struct BinaryContainer {
    compute: HomomorphicCompute,
    state: Vec<u8>,
    iteration: usize,
    inner_layer: Option<Box<BinaryContainer>>,
}

/// Represents the verification result for container interactions
#[derive(Debug)]
pub struct VerificationResult {
    /// Whether the current layer's relationship is valid
    pub valid: bool,
    /// Total number of layers in the verification chain
    pub layer_depth: usize,
    /// Whether all layers in the chain are valid
    pub all_layers_valid: bool,
}

impl BinaryContainer {
    /// Creates a new binary container with the specified number of layers
    pub fn new(mut rng: ChaCha20Rng, depth: usize) -> Result<Self> {
        // Generate initial key and state
        let mut key = vec![0u8; 32];
        let mut state = vec![0u8; 32];
        rng.fill_bytes(&mut key);
        rng.fill_bytes(&mut state);
        
        // Create package and add time steps
        let mut pkg = HolographicKeyPackage::new(key);
        pkg.add_time_step(&mut rng)?;
        
        // Create inner layer if depth > 1
        let inner_layer = if depth > 1 {
            let inner_rng = ChaCha20Rng::seed_from_u64(
                rng.next_u64().wrapping_add(depth as u64)
            );
            Some(Box::new(Self::new(inner_rng, depth - 1)?))
        } else {
            None
        };
        
        Ok(Self {
            compute: HomomorphicCompute::new(pkg)?,
            state,
            iteration: 0,
            inner_layer,
        })
    }
    
    /// Creates a new container by cloning the current one with a one-time pad
    pub fn clone_with_pad(&self, pad: &[u8]) -> Result<Self> {
        // Create a new container with state XORed with pad
        let mut cloned_state = self.state.clone();
        for (i, b) in cloned_state.iter_mut().enumerate() {
            *b ^= pad[i % pad.len()];
        }
        
        // Use a different seed for the clone to ensure unique evolution
        let rng = ChaCha20Rng::seed_from_u64((self.iteration as u64).wrapping_add(1));
        let mut container = Self::new(rng, self.depth())?;
        container.state = cloned_state;
        
        // Clone inner layer if it exists
        if let Some(inner) = &self.inner_layer {
            // Generate pad for inner layer using outer state
            let inner_pad: Vec<u8> = self.state.iter()
                .zip(pad.iter())
                .map(|(&a, &b)| a ^ b)
                .collect();
            container.inner_layer = Some(Box::new(inner.clone_with_pad(&inner_pad)?));
        }
        
        Ok(container)
    }
    
    /// Evolves the container's state by one iteration
    pub fn iterate(&mut self) -> Result<()> {
        // Generate iteration-specific entropy
        let mut entropy = vec![0u8; 32];
        ChaCha20Rng::seed_from_u64(
            (self.iteration as u64).wrapping_mul(0x517cc1b727220a95)
        ).fill_bytes(&mut entropy);
        
        // Load current state and entropy
        self.compute.load(0, &self.state)?;
        self.compute.load(1, &entropy)?;
        
        // Apply transformations
        self.compute.compute(Operation::Add, 0, 1)?;  // Add entropy
        self.compute.compute(Operation::Xor, 0, 1)?;  // Mix with entropy
        
        // Apply non-linear transformations
        let mut intermediate = self.compute.read(0)?;
        
        // Rotate bytes for diffusion
        if !intermediate.is_empty() {
            let mut rotated = vec![0u8; intermediate.len()];
            let len = intermediate.len();
            for i in 0..len {
                rotated[i] = intermediate[(i + 1) % len];
            }
            intermediate = rotated;
        }
        
        // Load rotated state and apply final transformations
        self.compute.load(0, &intermediate)?;
        self.compute.compute(Operation::Add, 0, 0)?;
        self.compute.compute(Operation::Xor, 0, 1)?;
        
        // Update state
        self.state = self.compute.read(0)?;
        self.iteration += 1;
        
        // Iterate inner layer if it exists
        if let Some(inner) = &mut self.inner_layer {
            inner.iterate()?;
        }
        
        Ok(())
    }
    
    /// Verifies the cryptographic relationship with another container
    pub fn interact(&mut self, other: &mut Self) -> Result<VerificationResult> {
        // Check if containers are in sync
        if self.iteration != other.iteration {
            return Ok(VerificationResult {
                valid: false,
                layer_depth: 1,
                all_layers_valid: false,
            });
        }
        
        // Load states
        self.compute.load(0, &self.state)?;
        self.compute.load(1, &other.state)?;
        
        // Compute interaction
        self.compute.compute(Operation::Xor, 0, 1)?;
        let interaction = self.compute.read(0)?;
        
        // Verify relationship is maintained using wrapping operations
        let expected_sum = interaction.iter()
            .fold(0u8, |acc, &x| acc.wrapping_add(x));
        let actual_sum = self.state.iter()
            .zip(other.state.iter())
            .map(|(a, b)| a ^ b)
            .fold(0u8, |acc, x| acc.wrapping_add(x));
        
        let current_valid = expected_sum == actual_sum;
        
        // Check inner layer if it exists and current layer is valid
        if current_valid {
            if let (Some(inner), Some(other_inner)) = (&mut self.inner_layer, &mut other.inner_layer) {
                let inner_result = inner.interact(other_inner)?;
                return Ok(VerificationResult {
                    valid: true,
                    layer_depth: inner_result.layer_depth + 1,
                    all_layers_valid: inner_result.all_layers_valid,
                });
            }
        }
        
        Ok(VerificationResult {
            valid: current_valid,
            layer_depth: 1,
            all_layers_valid: current_valid,
        })
    }
    
    /// Returns the current state of the container
    pub fn state(&self) -> &[u8] {
        &self.state
    }
    
    /// Returns the current iteration count
    pub fn iteration(&self) -> usize {
        self.iteration
    }
    
    /// Returns the total depth of nested layers
    pub fn depth(&self) -> usize {
        1 + self.inner_layer.as_ref().map_or(0, |inner| inner.depth())
    }
}
