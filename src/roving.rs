use crate::{error::FskcError, Result};
use nalgebra::DVector;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::collections::HashMap;

/// Represents a particle moving through the high-dimensional keyspace
#[derive(Debug, Clone)]
struct Particle {
    position: DVector<f64>,
    velocity: DVector<f64>,
}

/// Manages the selection of data points in high-dimensional space
#[derive(Debug)]
pub struct RovingSelector {
    dimension: usize,
    particles: Vec<Particle>,
    data_points: HashMap<usize, DVector<f64>>,
    rng: ChaCha20Rng,
}

impl RovingSelector {
    /// Creates a new RovingSelector with the specified parameters
    pub fn new(
        dimension: usize,
        num_particles: usize,
        seed: u64,
    ) -> Result<Self> {
        // Validate parameters
        if dimension == 0 {
            return Err(FskcError::GeometricError("Dimension cannot be zero".into()));
        }

        if num_particles == 0 {
            return Err(FskcError::InvalidParticles(0));
        }

        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut particles = Vec::with_capacity(num_particles);

        // Initialize particles with random positions and velocities
        for _ in 0..num_particles {
            let position = DVector::from_fn(dimension, |_, _| rng.gen_range(-1.0..1.0));
            let velocity = DVector::from_fn(dimension, |_, _| rng.gen_range(-0.1..0.1));
            particles.push(Particle { position, velocity });
        }

        Ok(Self {
            dimension,
            particles,
            data_points: HashMap::new(),
            rng,
        })
    }

    /// Maps data points into the high-dimensional space
    pub fn map_data(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(FskcError::InvalidDataSize(0));
        }

        // Clear existing data points
        self.data_points.clear();

        // Map each byte to a point in high-dimensional space
        for (i, &byte) in data.iter().enumerate() {
            let point = DVector::from_fn(self.dimension, |j, _| {
                let phase = (byte as f64 * (j + 1) as f64) / 256.0;
                phase.sin()
            });
            self.data_points.insert(i, point);
        }

        Ok(())
    }

    /// Moves particles through the space and selects nearby data points
    pub fn step(&mut self) -> Result<Vec<u8>> {
        let mut selected = Vec::new();
        
        // Update particle positions
        for particle in &mut self.particles {
            // Update position based on velocity
            particle.position += &particle.velocity;

            // Add some random movement
            let random_movement = DVector::from_fn(self.dimension, |_, _| {
                self.rng.gen_range(-0.05..0.05)
            });
            particle.velocity += random_movement;

            // Normalize velocity to prevent excessive speeds
            if particle.velocity.norm() > 1.0 {
                particle.velocity.normalize_mut();
            }

            // Find nearest data point
            // Find nearest data point and convert index to u8
            let nearest = self.data_points
                .iter()
                .map(|(&idx, point)| {
                    let distance = (point - &particle.position).norm();
                    (distance, idx)
                })
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            if let Some((_, idx)) = nearest {
                // Only use the index if it fits in a u8
                if idx < 256 {
                    selected.push(idx as u8);
                }
            }
        }

        Ok(selected)
    }

    /// Returns the current dimension of the space
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    /// Returns the number of particles
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roving_selector_creation() {
        let selector = RovingSelector::new(4, 10, 12345);
        assert!(selector.is_ok());
        
        let selector = selector.unwrap();
        assert_eq!(selector.dimension(), 4);
        assert_eq!(selector.particle_count(), 10);
    }

    #[test]
    fn test_data_mapping() {
        let mut selector = RovingSelector::new(4, 10, 12345).unwrap();
        let data = b"Test data".to_vec();
        
        assert!(selector.map_data(&data).is_ok());
        assert_eq!(selector.data_points.len(), data.len());
    }

    #[test]
    fn test_particle_movement() {
        let mut selector = RovingSelector::new(4, 10, 12345).unwrap();
        let data = b"Test data".to_vec();
        
        selector.map_data(&data).unwrap();
        let selected = selector.step().unwrap();
        
        assert!(!selected.is_empty());
        assert!(selected.iter().all(|&idx| (idx as usize) < data.len()));
    }
}
