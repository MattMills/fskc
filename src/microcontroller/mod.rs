pub mod error;
mod cpu;
mod instructions;
mod memory;

use crate::{Result, HomomorphicCompute};
pub use instructions::{Instruction, Register, MemoryAddress, StatusFlags};
pub use error::{CpuError, MemoryError};
use cpu::Cpu;

/// AVR-like microcontroller with homomorphic computation capabilities
pub struct Microcontroller {
    cpu: Cpu,
    word_size: usize,
}

impl Microcontroller {
    /// Create new microcontroller instance
    ///
    /// # Arguments
    /// * `compute` - Homomorphic compute engine
    /// * `memory_size` - Size of data memory in words
    /// * `word_size` - Size of each word in bytes (default: 32)
    pub fn new(compute: HomomorphicCompute, memory_size: usize) -> Result<Self> {
        Ok(Self {
            cpu: Cpu::new(compute, memory_size, 32),
            word_size: 32,
        })
    }
    
    /// Load program into memory
    ///
    /// # Arguments
    /// * `program` - Program bytes (each instruction is 2 bytes)
    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        self.cpu.load_program(program);
        Ok(())
    }
    
    /// Load data into memory
    ///
    /// # Arguments
    /// * `addr` - Memory address
    /// * `data` - Data bytes (must match word size)
    pub fn load_data(&mut self, addr: u16, data: &[u8]) -> Result<()> {
        if data.len() != self.word_size {
            return Err(MemoryError::InvalidSize {
                expected: self.word_size,
                got: data.len(),
            }.into());
        }
        self.cpu.load_data(MemoryAddress::new(addr), data)
    }
    
    /// Read memory value
    ///
    /// # Arguments
    /// * `addr` - Memory address
    pub fn get_memory(&self, addr: u16) -> Result<&[u8]> {
        self.cpu.read_memory(MemoryAddress::new(addr))
    }
    
    /// Execute loaded program
    pub fn execute(&mut self) -> Result<()> {
        self.cpu.execute()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HolographicKeyPackage;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    #[test]
    fn test_add_numbers() -> Result<()> {
        // Initialize compute engine
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let mut key = vec![0u8; 32];
        rng.fill_bytes(&mut key);
        let mut pkg = HolographicKeyPackage::new(key);
        pkg.add_time_step(&mut rng)?;
        let compute = HomomorphicCompute::new(pkg)?;
        
        // Create microcontroller
        let mut mc = Microcontroller::new(compute, 256)?;
        
        // Load test program: Add mem[0] and mem[1], store in mem[2]
        let program = vec![
            0x80, 0x01,  // LD R0, mem[0]
            0x80, 0x12,  // LD R1, mem[1]
            0x03, 0x12,  // ADD R2 <- R0 + R1
            0x82, 0x22,  // ST mem[2], R2
            0xFF, 0x00,  // HALT
        ];
        
        mc.load_program(&program)?;
        
        // Load test data
        let data1 = vec![42u8; 32];
        let data2 = vec![24u8; 32];
        
        mc.load_data(0, &data1)?;
        mc.load_data(1, &data2)?;
        
        // Execute program
        mc.execute()?;
        
        // Verify result (should be normalized form of 66)
        let result = mc.get_memory(2)?;
        assert!(!result.iter().all(|&x| x == 0));
        
        Ok(())
    }
}
