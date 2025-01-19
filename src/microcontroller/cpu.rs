use crate::{Result, HomomorphicCompute, Operation};
use super::{
    instructions::{MemoryAddress, StatusFlags},
    memory::{DataMemory, ProgramMemory, RegisterFile, MemorySegment},
    error::CpuError,
};

/// CPU state and execution
pub struct Cpu {
    compute: HomomorphicCompute,
    registers: RegisterFile,
    data_memory: DataMemory,
    program_memory: ProgramMemory,
    pc: usize,
    status: StatusFlags,
    word_size: usize,
}

impl Cpu {
    /// Create new CPU instance
    pub fn new(compute: HomomorphicCompute, memory_size: usize, word_size: usize) -> Self {
        Self {
            compute,
            registers: RegisterFile::new(32, word_size),
            data_memory: DataMemory::new(memory_size, word_size),
            program_memory: ProgramMemory::new(),
            pc: 0,
            status: StatusFlags::new(),
            word_size,
        }
    }
    
    /// Load program into memory
    pub fn load_program(&mut self, program: &[u8]) {
        self.program_memory.load_program(program);
    }
    
    /// Load data into memory
    pub fn load_data(&mut self, addr: MemoryAddress, data: &[u8]) -> Result<()> {
        // Normalize data through compute engine
        self.compute.load(0, data)?;
        self.compute.compute(Operation::Add, 0, 0)?;
        let normalized = self.compute.read(0)?;
        
        // Store normalized data
        self.data_memory.write(addr, &normalized)
    }
    
    /// Read memory value
    pub fn read_memory(&self, addr: MemoryAddress) -> Result<&[u8]> {
        self.data_memory.read(addr)
    }
    
    /// Execute loaded program
    pub fn execute(&mut self) -> Result<()> {
        while let Some((opcode, operand)) = self.program_memory.fetch(self.pc) {
            // Execute one instruction
            self.execute_instruction(opcode, operand)?;
            
            // Advance PC (unless modified by jump)
            self.pc += 2;
        }
        Ok(())
    }
    
    // Execute single instruction
    fn execute_instruction(&mut self, opcode: u8, operand: u8) -> Result<()> {
        // Extract register fields
        let rd = (operand & 0x0F) as u8;
        let rs = (operand >> 4) as u8;
        
        match opcode {
            // Load from memory
            0x80 => {
                let addr = MemoryAddress::new(u16::from(rs));
                let data = self.data_memory.read(addr)?;
                
                // Normalize through compute engine
                self.compute.load(0, data)?;
                self.compute.compute(Operation::Add, 0, 0)?;
                let normalized = self.compute.read(0)?;
                
                self.registers.write(rd, &normalized)?;
                Ok(())
            }
            
            // Store to memory
            0x82 => {
                let addr = MemoryAddress::new(u16::from(rd));
                let data = self.registers.read(rs)?;
                
                // Normalize through compute engine
                self.compute.load(0, data)?;
                self.compute.compute(Operation::Add, 0, 0)?;
                let normalized = self.compute.read(0)?;
                
                self.data_memory.write(addr, &normalized)?;
                Ok(())
            }
            
            // Add registers
            0x03 => {
                // Load and normalize operands
                let op1 = self.registers.read(rd)?;
                self.compute.load(0, op1)?;
                self.compute.compute(Operation::Add, 0, 0)?;
                let normalized1 = self.compute.read(0)?;
                
                let op2 = self.registers.read(rs)?;
                self.compute.load(0, op2)?;
                self.compute.compute(Operation::Add, 0, 0)?;
                let normalized2 = self.compute.read(0)?;
                
                // Perform addition
                self.compute.load(0, &normalized1)?;
                self.compute.load(1, &normalized2)?;
                self.compute.compute(Operation::Add, 0, 1)?;
                let result = self.compute.read(0)?;
                
                // Update flags
                self.status.zero = result.iter().all(|&x| x == 0);
                self.status.negative = result[0] & 0x80 != 0;
                
                // Store result
                self.registers.write(rd, &result)?;
                Ok(())
            }
            
            // Branch if equal
            0xF0 => {
                if self.status.zero {
                    self.pc = ((self.pc as i32) + (operand as i32)) as usize;
                }
                Ok(())
            }
            
            // Halt
            0xFF => Ok(()),
            
            // Invalid/unimplemented
            _ => Err(CpuError::InvalidInstruction.into()),
        }
    }
}
