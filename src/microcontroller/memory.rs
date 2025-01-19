use crate::{Result, FskcError};
use super::{
    instructions::MemoryAddress,
    error::MemoryError,
};

/// Memory segment trait
pub trait MemorySegment {
    /// Read data at address
    fn read(&self, addr: MemoryAddress) -> Result<&[u8]>;
    
    /// Write data at address
    fn write(&mut self, addr: MemoryAddress, data: &[u8]) -> Result<()>;
    
    /// Get segment size
    fn size(&self) -> usize;
}

/// Data memory implementation
pub struct DataMemory {
    data: Vec<Vec<u8>>,
    word_size: usize,
}

impl DataMemory {
    /// Create new data memory
    pub fn new(size: usize, word_size: usize) -> Self {
        Self {
            data: vec![vec![0u8; word_size]; size],
            word_size,
        }
    }
}

impl MemorySegment for DataMemory {
    fn read(&self, addr: MemoryAddress) -> Result<&[u8]> {
        self.data.get(addr.value())
            .map(|v| v.as_slice())
            .ok_or_else(|| MemoryError::OutOfBounds(addr.value()).into())
    }
    
    fn write(&mut self, addr: MemoryAddress, data: &[u8]) -> Result<()> {
        if data.len() != self.word_size {
            return Err(MemoryError::InvalidSize {
                expected: self.word_size,
                got: data.len(),
            }.into());
        }
        
        let slot = self.data.get_mut(addr.value()).ok_or_else(|| {
            FskcError::from(MemoryError::OutOfBounds(addr.value()))
        })?;
        
        slot.copy_from_slice(data);
        Ok(())
    }
    
    fn size(&self) -> usize {
        self.data.len()
    }
}

/// Program memory implementation
pub struct ProgramMemory {
    instructions: Vec<u8>,
}

impl ProgramMemory {
    /// Create new program memory
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }
    
    /// Load program into memory
    pub fn load_program(&mut self, program: &[u8]) {
        self.instructions = program.to_vec();
    }
    
    /// Get instruction bytes at PC
    pub fn fetch(&self, pc: usize) -> Option<(u8, u8)> {
        if pc + 1 >= self.instructions.len() {
            None
        } else {
            Some((self.instructions[pc], self.instructions[pc + 1]))
        }
    }
    
    /// Get program size in bytes
    pub fn len(&self) -> usize {
        self.instructions.len()
    }
    
    /// Check if program memory is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

/// Register file implementation
pub struct RegisterFile {
    registers: Vec<Vec<u8>>,
    word_size: usize,
}

impl RegisterFile {
    /// Create new register file
    pub fn new(num_registers: usize, word_size: usize) -> Self {
        Self {
            registers: vec![vec![0u8; word_size]; num_registers],
            word_size,
        }
    }
    
    /// Read register value
    pub fn read(&self, reg: u8) -> Result<&[u8]> {
        self.registers.get(reg as usize)
            .map(|v| v.as_slice())
            .ok_or_else(|| MemoryError::OutOfBounds(reg as usize).into())
    }
    
    /// Write register value
    pub fn write(&mut self, reg: u8, data: &[u8]) -> Result<()> {
        if data.len() != self.word_size {
            return Err(MemoryError::InvalidSize {
                expected: self.word_size,
                got: data.len(),
            }.into());
        }
        
        let slot = self.registers.get_mut(reg as usize).ok_or_else(|| {
            FskcError::from(MemoryError::OutOfBounds(reg as usize))
        })?;
        
        slot.copy_from_slice(data);
        Ok(())
    }
}
