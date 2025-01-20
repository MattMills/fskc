use crate::{Result, HomomorphicCompute};
use super::instructions::{Instruction, MemoryAddress, StatusFlags};
use super::memory::Memory;
use super::error::CpuError;

/// AVR-like CPU with homomorphic computation capabilities
pub struct Cpu {
    /// Program counter
    pc: usize,
    /// Status flags
    flags: StatusFlags,
    /// Compute engine
    compute: HomomorphicCompute,
    /// Memory
    memory: Memory,
}

impl Cpu {
    /// Create new CPU instance
    pub fn new(compute: HomomorphicCompute, memory_size: usize, _word_size: usize) -> Self {
        Self {
            pc: 0,
            flags: StatusFlags::new(),
            compute,
            memory: Memory::new(memory_size),
        }
    }

    /// Get current program counter
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Load program into memory
    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        // Write program bytes directly to program memory
        self.memory.write_program(program)?;
        Ok(())
    }

    /// Load data into memory
    pub fn load_data(&mut self, addr: MemoryAddress, data: &[u8]) -> Result<()> {
        self.memory.write_bytes(addr.value(), data)
    }

    /// Read from memory
    pub fn read_memory(&self, addr: MemoryAddress) -> Result<&[u8]> {
        self.memory.read_bytes(addr.value())
    }

    /// Execute loaded program
    pub fn execute(&mut self) -> Result<()> {
        loop {
            // Read instruction bytes
            let bytes = self.memory.read_bytes(self.pc)?;
            let opcode = bytes[0];
            let operand = bytes[1];

            // Decode and execute
            if let Some(instr) = Instruction::decode(opcode, operand) {
                match instr {
                    Instruction::Halt => break,
                    _ => {
                        self.execute_instruction(instr)?;
                    }
                }
                self.pc += 2; // Move to next instruction after execution
            } else {
                return Err(CpuError::InvalidInstruction.into());
            }
        }
        Ok(())
    }

    /// Execute single instruction
    fn execute_instruction(&mut self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::Load(rd, addr) => {
                // Load from data memory into register
                let data = self.memory.read_bytes(addr.value())?.to_vec();
                self.memory.write_bytes(rd.index() + 0x100, &data)?;
            }
            Instruction::Store(addr, rs) => {
                // Store from register to data memory
                let data = self.memory.read_bytes(rs.index() + 0x100)?.to_vec();
                self.memory.write_bytes(addr.value(), &data)?;
            }
            Instruction::Add(rd, rs1, rs2) => {
                // Read operands from registers
                let op1 = self.memory.read_bytes(rs1.index() + 0x100)?.to_vec();
                let op2 = self.memory.read_bytes(rs2.index() + 0x100)?.to_vec();
                
                // Perform addition in homomorphic space
                self.compute.load(0, &op1)?;
                self.compute.load(1, &op2)?;
                self.compute.compute(crate::Operation::Add, 0, 1)?;
                let result = self.compute.read(0)?;
                
                // Store result in destination register
                self.memory.write_bytes(rd.index() + 0x100, &result)?;
            }
            Instruction::Xor(rd, rs1, rs2) => {
                // Read operands from registers
                let op1 = self.memory.read_bytes(rs1.index() + 0x100)?.to_vec();
                let op2 = self.memory.read_bytes(rs2.index() + 0x100)?.to_vec();
                
                // Perform XOR in homomorphic space
                self.compute.load(0, &op1)?;
                self.compute.load(1, &op2)?;
                self.compute.compute(crate::Operation::Xor, 0, 1)?;
                let result = self.compute.read(0)?;
                
                // Store result in destination register
                self.memory.write_bytes(rd.index() + 0x100, &result)?;
            }
            Instruction::Jump(addr) => {
                self.pc = addr.value();
            }
            Instruction::BranchEq(addr) => {
                if self.flags.zero {
                    self.pc = addr.value();
                }
            }
            _ => {} // Other instructions not implemented yet
        }
        Ok(())
    }
}
