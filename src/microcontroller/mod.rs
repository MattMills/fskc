pub mod error;
pub mod cpu;
pub mod instructions;
pub mod memory;

use crate::{Result, HomomorphicCompute};
pub use instructions::{Instruction, Register, MemoryAddress, StatusFlags};
pub use error::{CpuError, MemoryError};
pub use cpu::Cpu;
pub use memory::{Memory, MemorySegment};

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
        self.cpu.load_program(program)
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

    /// Get current program counter
    pub fn pc(&self) -> usize {
        self.cpu.pc()
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
        
        // Load test program: Add data mem[0] and mem[1], store in mem[2]
        let program = vec![
            0x80, 0x00,  // LOAD R0, [0x400]   ; Load from data mem[0] into R0
            0x81, 0x01,  // LOAD R1, [0x401]   ; Load from data mem[1] into R1
            0x03, 0x14,  // ADD R1, R0, R1     ; Add R0 and R1, store in R1 (rd=1, rs1=0, rs2=1)
            0x91, 0x02,  // STORE [0x402], R1  ; Store R1 to data mem[2]
            0xFF, 0x00,  // HALT
        ];
        
        mc.load_program(&program)?;
        
        // Load test data into data memory (offset by 0x400)
        let data1 = vec![42u8; 32];
        let data2 = vec![24u8; 32];
        
        mc.load_data(0x400, &data1)?;
        mc.load_data(0x401, &data2)?;
        
        // Execute program
        mc.execute()?;
        
        // Verify result at data memory offset (should be normalized form of 66)
        let result = mc.get_memory(0x402)?;
        assert!(!result.iter().all(|&x| x == 0));
        
        Ok(())
    }
}
