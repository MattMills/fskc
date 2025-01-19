use crate::{
    Result,
    HomomorphicCompute,
    Operation,
};

/// Represents a RISC-like instruction
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    // Data movement
    Load(u8, u8),    // Load from memory[src] to register[dst]
    Store(u8, u8),   // Store register[src] to memory[dst]
    Move(u8, u8),    // Move register[src] to register[dst]
    
    // Arithmetic
    Add(u8, u8, u8), // Add register[src1] + register[src2] -> register[dst]
    Sub(u8, u8, u8), // Sub register[src1] - register[src2] -> register[dst]
    Xor(u8, u8, u8), // Xor register[src1] ^ register[src2] -> register[dst]
    
    // Control flow
    Jump(u8),        // Jump to memory[addr]
    JumpIf(u8, u8),  // Jump to memory[addr] if register[cond] != 0
    Halt,            // Stop execution
}

/// Virtual machine state
pub struct VirtualMachine {
    compute: HomomorphicCompute,
    registers: Vec<Vec<u8>>,    // General purpose registers
    program: Vec<Vec<u8>>,      // Program instructions
    memory: Vec<Vec<u8>>,       // Data memory
    pc: usize,                  // Program counter
}

impl VirtualMachine {
    /// Creates a new virtual machine instance
    pub fn new(compute: HomomorphicCompute, memory_size: usize) -> Result<Self> {
        Ok(Self {
            compute,
            registers: vec![vec![0u8; 32]; 16],  // 16 registers of 32 bytes each
            program: Vec::new(),                 // Program memory
            memory: vec![vec![0u8; 32]; memory_size], // Data memory
            pc: 0,
        })
    }
    
    /// Loads a program into program memory
    pub fn load_program(&mut self, program: &[Vec<u8>]) -> Result<()> {
        self.program = program.to_vec();
        Ok(())
    }
    
    /// Executes the loaded program
    pub fn execute(&mut self) -> Result<()> {
        loop {
            // Fetch instruction
            if self.pc >= self.program.len() {
                break;  // End of program
            }
            let instruction = self.decode_instruction(&self.program[self.pc])?;
            
            // Execute instruction
            match instruction {
                Instruction::Load(src, dst) => {
                    // Load memory into compute engine
                    self.compute.load(0, &self.memory[src as usize])?;
                    // Apply identity operation to normalize the data
                    self.compute.compute(Operation::Add, 0, 0)?;
                    // Store result in register
                    self.registers[dst as usize] = self.compute.read(0)?;
                }
                
                Instruction::Store(src, dst) => {
                    // Load source register into compute engine
                    self.compute.load(0, &self.registers[src as usize])?;
                    // Apply identity operation to normalize the data
                    self.compute.compute(Operation::Add, 0, 0)?;
                    // Store result in memory
                    self.memory[dst as usize] = self.compute.read(0)?;
                }
                
                Instruction::Move(src, dst) => {
                    // Load source register into compute engine
                    self.compute.load(0, &self.registers[src as usize])?;
                    // Apply identity operation to normalize the data
                    self.compute.compute(Operation::Add, 0, 0)?;
                    // Store result in destination register
                    self.registers[dst as usize] = self.compute.read(0)?;
                }
                
                Instruction::Add(src1, src2, dst) => {
                    // Load first operand and normalize
                    self.compute.load(0, &self.registers[src1 as usize])?;
                    self.compute.compute(Operation::Add, 0, 0)?;
                    let op1 = self.compute.read(0)?;
                    
                    // Load second operand and normalize
                    self.compute.load(0, &self.registers[src2 as usize])?;
                    self.compute.compute(Operation::Add, 0, 0)?;
                    let op2 = self.compute.read(0)?;
                    
                    // Perform addition
                    self.compute.load(0, &op1)?;
                    self.compute.load(1, &op2)?;
                    self.compute.compute(Operation::Add, 0, 1)?;
                    
                    // Store result
                    self.registers[dst as usize] = self.compute.read(0)?;
                }
                
                Instruction::Sub(src1, src2, dst) => {
                    // Load first operand and normalize
                    self.compute.load(0, &self.registers[src1 as usize])?;
                    self.compute.compute(Operation::Add, 0, 0)?;
                    let op1 = self.compute.read(0)?;
                    
                    // Load second operand and normalize
                    self.compute.load(0, &self.registers[src2 as usize])?;
                    self.compute.compute(Operation::Add, 0, 0)?;
                    let op2 = self.compute.read(0)?;
                    
                    // Perform subtraction (Add + Negate)
                    self.compute.load(0, &op1)?;
                    self.compute.load(1, &op2)?;
                    self.compute.compute(Operation::Add, 0, 1)?;
                    self.compute.compute(Operation::Xor, 0, 0)?;  // Negate result
                    
                    // Store result
                    self.registers[dst as usize] = self.compute.read(0)?;
                }
                
                Instruction::Xor(src1, src2, dst) => {
                    // Load first operand and normalize
                    self.compute.load(0, &self.registers[src1 as usize])?;
                    self.compute.compute(Operation::Add, 0, 0)?;
                    let op1 = self.compute.read(0)?;
                    
                    // Load second operand and normalize
                    self.compute.load(0, &self.registers[src2 as usize])?;
                    self.compute.compute(Operation::Add, 0, 0)?;
                    let op2 = self.compute.read(0)?;
                    
                    // Perform XOR
                    self.compute.load(0, &op1)?;
                    self.compute.load(1, &op2)?;
                    self.compute.compute(Operation::Xor, 0, 1)?;
                    
                    // Store result
                    self.registers[dst as usize] = self.compute.read(0)?;
                }
                
                Instruction::Jump(addr) => {
                    self.pc = addr as usize;
                    continue;
                }
                
                Instruction::JumpIf(cond, addr) => {
                    if !self.registers[cond as usize].iter().all(|&x| x == 0) {
                        self.pc = addr as usize;
                        continue;
                    }
                }
                
                Instruction::Halt => break,
            }
            
            self.pc += 1;
        }
        
        Ok(())
    }
    
    /// Returns the current state of a register
    pub fn get_register(&self, reg: u8) -> &[u8] {
        &self.registers[reg as usize]
    }
    
    /// Returns the current state of a memory location
    pub fn get_memory(&self, addr: u8) -> &[u8] {
        &self.memory[addr as usize]
    }
    
    /// Returns a mutable reference to a memory location
    pub fn get_memory_mut(&mut self, addr: u8) -> &mut Vec<u8> {
        &mut self.memory[addr as usize]
    }
    
    /// Loads normalized data into memory
    pub fn load_data(&mut self, addr: u8, data: &[u8]) -> Result<()> {
        // Load data into compute engine and normalize
        self.compute.load(0, data)?;
        self.compute.compute(Operation::Add, 0, 0)?;
        // Store normalized data in memory
        self.memory[addr as usize] = self.compute.read(0)?;
        Ok(())
    }
    
    // Internal helper to decode instruction bytes into enum
    fn decode_instruction(&self, bytes: &[u8]) -> Result<Instruction> {
        // Simple encoding: first byte is opcode, rest are parameters
        match bytes[0] {
            0 => Ok(Instruction::Load(bytes[1], bytes[2])),
            1 => Ok(Instruction::Store(bytes[1], bytes[2])),
            2 => Ok(Instruction::Move(bytes[1], bytes[2])),
            3 => Ok(Instruction::Add(bytes[1], bytes[2], bytes[3])),
            4 => Ok(Instruction::Sub(bytes[1], bytes[2], bytes[3])),
            5 => Ok(Instruction::Xor(bytes[1], bytes[2], bytes[3])),
            6 => Ok(Instruction::Jump(bytes[1])),
            7 => Ok(Instruction::JumpIf(bytes[1], bytes[2])),
            8 => Ok(Instruction::Halt),
            _ => Ok(Instruction::Halt), // Invalid opcode treated as halt
        }
    }
}
