use crate::Result;
use super::error::MemoryError;

/// Memory segment trait
pub trait MemorySegment {
    /// Read value at offset
    fn read(&self, offset: usize) -> Result<&[u8]>;
    /// Write value at offset
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<()>;
    /// Get segment size
    fn size(&self) -> usize;
}

/// Program memory segment
#[derive(Debug)]
pub struct ProgramMemory {
    data: Vec<u8>,
}

/// Register memory segment
#[derive(Debug)]
pub struct RegisterMemory {
    data: Vec<Vec<u8>>,
    word_size: usize,
}

/// Data memory segment
#[derive(Debug)]
pub struct DataMemory {
    data: Vec<Vec<u8>>,
    word_size: usize,
}

/// Memory management unit
#[derive(Debug)]
pub struct Memory {
    program: ProgramMemory,    // 0x000-0x0FF: Program memory
    registers: RegisterMemory, // 0x100-0x1FF: Register file
    data: DataMemory,         // 0x400+: Data memory
}

impl ProgramMemory {
    /// Create new program memory segment
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0; size],
        }
    }

    /// Write byte at address
    pub fn write_byte(&mut self, addr: usize, byte: u8) -> Result<()> {
        if addr >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr,
                size: self.data.len(),
            }.into());
        }
        self.data[addr] = byte;
        Ok(())
    }

    /// Read byte at address
    pub fn read_byte(&self, addr: usize) -> Result<u8> {
        if addr >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr,
                size: self.data.len(),
            }.into());
        }
        Ok(self.data[addr])
    }

    /// Get program memory size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if program memory is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl RegisterMemory {
    /// Create new register memory segment
    pub fn new(size: usize, word_size: usize) -> Self {
        Self {
            data: vec![vec![0; word_size]; size],
            word_size,
        }
    }
}

impl DataMemory {
    /// Create new data memory segment
    pub fn new(size: usize, word_size: usize) -> Self {
        Self {
            data: vec![vec![0; word_size]; size],
            word_size,
        }
    }
}

impl Memory {
    /// Create new memory instance
    pub fn new(size: usize) -> Self {
        Self {
            program: ProgramMemory::new(size * 2),     // 2 bytes per instruction
            registers: RegisterMemory::new(32, 32),    // 32 registers, 32 bytes each
            data: DataMemory::new(size, 32),          // Data memory, 32 bytes per word
        }
    }

    /// Write program bytes directly to program memory
    pub fn write_program(&mut self, program: &[u8]) -> Result<()> {
        if program.len() > self.program.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr: program.len(),
                size: self.program.data.len(),
            }.into());
        }
        self.program.data[..program.len()].copy_from_slice(program);
        Ok(())
    }

    /// Read program byte at address
    pub fn read_program_byte(&self, addr: usize) -> Result<u8> {
        if addr >= self.program.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr,
                size: self.program.data.len(),
            }.into());
        }
        Ok(self.program.data[addr])
    }

    /// Write bytes to memory
    pub fn write_bytes(&mut self, addr: usize, data: &[u8]) -> Result<()> {
        if addr < 0x100 {
            // Program memory (0x000-0x0FF)
            if addr + data.len() > self.program.data.len() {
                return Err(MemoryError::OutOfBounds {
                    addr,
                    size: self.program.data.len(),
                }.into());
            }
            self.program.data[addr..addr+data.len()].copy_from_slice(data);
        } else if addr < 0x200 {
            // Register file (0x100-0x1FF)
            let reg_addr = addr - 0x100;
            if reg_addr >= self.registers.data.len() {
                return Err(MemoryError::OutOfBounds {
                    addr: reg_addr,
                    size: self.registers.data.len(),
                }.into());
            }
            if data.len() != 32 {
                return Err(MemoryError::InvalidSize {
                    expected: 32,
                    got: data.len(),
                }.into());
            }
            self.registers.data[reg_addr].copy_from_slice(data);
        } else {
            // Data memory (0x400+)
            let data_addr = addr - 0x400;
            if data_addr >= self.data.data.len() {
                return Err(MemoryError::OutOfBounds {
                    addr: data_addr,
                    size: self.data.data.len(),
                }.into());
            }
            if data.len() != 32 {
                return Err(MemoryError::InvalidSize {
                    expected: 32,
                    got: data.len(),
                }.into());
            }
            self.data.data[data_addr].copy_from_slice(data);
        }
        Ok(())
    }

    /// Read bytes from memory
    pub fn read_bytes(&self, addr: usize) -> Result<&[u8]> {
        if addr < 0x100 {
            // Program memory (0x000-0x0FF)
            if addr + 1 >= self.program.data.len() {
                return Err(MemoryError::OutOfBounds {
                    addr,
                    size: self.program.data.len(),
                }.into());
            }
            Ok(&self.program.data[addr..addr+2])
        } else if addr < 0x200 {
            // Register file (0x100-0x1FF)
            let reg_addr = addr - 0x100;
            if reg_addr >= self.registers.data.len() {
                return Err(MemoryError::OutOfBounds {
                    addr: reg_addr,
                    size: self.registers.data.len(),
                }.into());
            }
            Ok(&self.registers.data[reg_addr])
        } else {
            // Data memory (0x400+)
            let data_addr = addr - 0x400;
            if data_addr >= self.data.data.len() {
                return Err(MemoryError::OutOfBounds {
                    addr: data_addr,
                    size: self.data.data.len(),
                }.into());
            }
            Ok(&self.data.data[data_addr])
        }
    }
}

impl MemorySegment for ProgramMemory {
    fn read(&self, offset: usize) -> Result<&[u8]> {
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr: offset,
                size: self.data.len(),
            }.into());
        }
        Ok(&self.data[offset..offset+1])
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr: offset,
                size: self.data.len(),
            }.into());
        }
        self.data[offset] = data[0];
        Ok(())
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl MemorySegment for RegisterMemory {
    fn read(&self, offset: usize) -> Result<&[u8]> {
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr: offset,
                size: self.data.len(),
            }.into());
        }
        Ok(&self.data[offset])
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr: offset,
                size: self.data.len(),
            }.into());
        }
        if data.len() != self.word_size {
            return Err(MemoryError::InvalidSize {
                expected: self.word_size,
                got: data.len(),
            }.into());
        }
        self.data[offset].copy_from_slice(data);
        Ok(())
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

impl MemorySegment for DataMemory {
    fn read(&self, offset: usize) -> Result<&[u8]> {
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr: offset,
                size: self.data.len(),
            }.into());
        }
        Ok(&self.data[offset])
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if offset >= self.data.len() {
            return Err(MemoryError::OutOfBounds {
                addr: offset,
                size: self.data.len(),
            }.into());
        }
        if data.len() != self.word_size {
            return Err(MemoryError::InvalidSize {
                expected: self.word_size,
                got: data.len(),
            }.into());
        }
        self.data[offset].copy_from_slice(data);
        Ok(())
    }

    fn size(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_memory() -> Result<()> {
        let mut mem = ProgramMemory::new(256);
        
        // Test byte write/read
        mem.write_byte(0, 0x42)?;
        assert_eq!(mem.read_byte(0)?, 0x42);
        
        // Test bounds
        assert!(mem.write_byte(256, 0x42).is_err());
        assert!(mem.read_byte(256).is_err());
        
        Ok(())
    }

    #[test]
    fn test_data_memory() -> Result<()> {
        let mut mem = DataMemory::new(256, 32);
        
        // Test word write/read
        let data = vec![0x42; 32];
        mem.write(0, &data)?;
        assert_eq!(mem.read(0)?, &data[..]);
        
        // Test bounds and size validation
        assert!(mem.write(256, &data).is_err());
        assert!(mem.write(0, &vec![0x42; 16]).is_err());
        
        Ok(())
    }

    #[test]
    fn test_memory_layout() -> Result<()> {
        let mut mem = Memory::new(256);
        
        // Test endianness preservation in program memory
        let test_val: u16 = 0x4217; // Test value with different high/low bytes
        let bytes = test_val.to_le_bytes(); // Use little-endian for test
        mem.write_bytes(0, &bytes)?;
        let read_bytes = mem.read_bytes(0)?;
        assert_eq!(read_bytes, &bytes, "Program memory should preserve byte order");
        let recovered_val = u16::from_le_bytes([read_bytes[0], read_bytes[1]]);
        assert_eq!(recovered_val, test_val, "Value should be preserved through memory operations");
        
        // Test bit pattern preservation in register file
        let bit_pattern = vec![
            0b10101010, // Alternating bits
            0b11110000, // High/low nibbles
            0b00001111, // Low/high nibbles
            0xFF,       // All ones
            0x00,       // All zeros
        ];
        let mut reg_data = vec![0; 32];
        reg_data[..5].copy_from_slice(&bit_pattern);
        
        mem.write_bytes(0x100, &reg_data)?;
        let read_reg = mem.read_bytes(0x100)?;
        assert_eq!(&read_reg[..5], &bit_pattern, "Register should preserve bit patterns");
        
        // Test LSB/MSB handling in data memory
        let mut data = vec![0; 32];
        // Test different byte patterns
        data[0] = 0x80; // MSB set
        data[1] = 0x01; // LSB set
        data[2] = 0x55; // Alternating bits starting with LSB
        data[3] = 0xAA; // Alternating bits starting with MSB
        
        mem.write_bytes(0x400, &data)?;
        let read_data = mem.read_bytes(0x400)?;
        
        // Verify bit patterns are preserved
        assert_eq!(read_data[0], 0x80, "MSB should be preserved");
        assert_eq!(read_data[1], 0x01, "LSB should be preserved");
        assert_eq!(read_data[2], 0x55, "Alternating pattern (LSB first) should be preserved");
        assert_eq!(read_data[3], 0xAA, "Alternating pattern (MSB first) should be preserved");
        
        // Test bounds
        assert!(mem.write_bytes(0x800, &data).is_err());
        assert!(mem.write_bytes(0x400, &[0x42]).is_err()); // Wrong size for data memory
        
        Ok(())
    }
}
