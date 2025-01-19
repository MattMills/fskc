/// AVR-like instruction set
#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    // Arithmetic & Logic
    Add(Register, Register, Register), // Add: Rd ← Rs1 + Rs2
    Sub(Register, Register, Register), // Sub: Rd ← Rs1 - Rs2
    And(Register, Register, Register), // And: Rd ← Rs1 ∧ Rs2
    Or(Register, Register, Register),  // Or:  Rd ← Rs1 ∨ Rs2
    Xor(Register, Register, Register), // Xor: Rd ← Rs1 ⊕ Rs2
    
    // Data Transfer
    Load(Register, MemoryAddress),     // Load:  Rd ← Mem[addr]
    Store(MemoryAddress, Register),    // Store: Mem[addr] ← Rs
    Move(Register, Register),          // Move:  Rd ← Rs
    
    // Control Flow
    Jump(ProgramCounter),              // Jump to address
    BranchEq(ProgramCounter),          // Branch if equal (Z=1)
    BranchNe(ProgramCounter),          // Branch if not equal (Z=0)
    
    // System
    Nop,                               // No operation
    Halt,                              // Stop execution
}

/// Register identifier (0-31)
#[derive(Debug, Clone, Copy)]
pub struct Register(pub u8);

impl Register {
    /// Create a new register identifier
    pub fn new(id: u8) -> Option<Self> {
        if id < 32 {
            Some(Register(id))
        } else {
            None
        }
    }
    
    /// Get register index
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

/// Memory address
#[derive(Debug, Clone, Copy)]
pub struct MemoryAddress(pub u16);

impl MemoryAddress {
    /// Create a new memory address
    pub fn new(addr: u16) -> Self {
        MemoryAddress(addr)
    }
    
    /// Get address value
    pub fn value(&self) -> usize {
        self.0 as usize
    }
}

/// Program counter
#[derive(Debug, Clone, Copy)]
pub struct ProgramCounter(pub i16);

impl ProgramCounter {
    /// Create a new program counter value
    pub fn new(offset: i16) -> Self {
        ProgramCounter(offset)
    }
    
    /// Get counter value
    pub fn value(&self) -> i16 {
        self.0
    }
}

/// Status register flags
#[derive(Debug, Clone, Copy)]
pub struct StatusFlags {
    pub zero: bool,      // Z: Zero flag
    pub negative: bool,  // N: Negative flag
}

impl StatusFlags {
    /// Create new status flags
    pub fn new() -> Self {
        StatusFlags {
            zero: false,
            negative: false,
        }
    }
}

/// Instruction decoder
pub trait InstructionDecoder {
    /// Decode raw bytes into instruction
    fn decode(&self, opcode: u8, operand: u8) -> Option<Instruction>;
}

/// Default AVR-like instruction decoder
pub struct AvrDecoder;

impl InstructionDecoder for AvrDecoder {
    fn decode(&self, opcode: u8, operand: u8) -> Option<Instruction> {
        // Extract register and address fields
        let rd = Register::new(operand & 0x0F)?;
        let rs = Register::new(operand >> 4)?;
        let addr = MemoryAddress::new(u16::from(operand >> 4));
        let offset = ProgramCounter::new(operand as i16);
        
        match opcode {
            0x80 => Some(Instruction::Load(rd, addr)),
            0x82 => Some(Instruction::Store(addr, rd)),
            0x03 => Some(Instruction::Add(rd, rs, rs)), // rs used twice since we only have 2 fields
            0xF0 => Some(Instruction::BranchEq(offset)),
            0xFF => Some(Instruction::Halt),
            _ => Some(Instruction::Nop),
        }
    }
}
