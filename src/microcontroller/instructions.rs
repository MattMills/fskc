/// AVR-like instruction set
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    /// Load from memory into register
    Load(Register, MemoryAddress),
    /// Store register to memory
    Store(MemoryAddress, Register),
    /// Add two registers
    Add(Register, Register, Register),
    /// Subtract two registers
    Sub(Register, Register, Register),
    /// AND two registers
    And(Register, Register, Register),
    /// OR two registers
    Or(Register, Register, Register),
    /// XOR two registers
    Xor(Register, Register, Register),
    /// Jump to address
    Jump(MemoryAddress),
    /// Branch if equal
    BranchEq(MemoryAddress),
    /// No operation
    Nop,
    /// Halt execution
    Halt,
}

impl Instruction {
    /// Encode instruction as bytes in little-endian format
    pub fn encode(&self) -> [u8; 2] {
        match self {
            Instruction::Load(reg, addr) => [0x80 | reg.index() as u8, (addr.value() & 0xFF) as u8],
            Instruction::Store(addr, reg) => [0x90 | reg.index() as u8, (addr.value() & 0xFF) as u8],
            Instruction::Add(rd, rs1, rs2) => [0x03, ((rd.index() & 0x03) << 4 | (rs1.index() & 0x03) << 2 | (rs2.index() & 0x03)) as u8],
            Instruction::Sub(rd, rs1, _rs2) => [0x04, (rd.index() << 4 | rs1.index()) as u8],
            Instruction::And(rd, rs1, _rs2) => [0x05, (rd.index() << 4 | rs1.index()) as u8],
            Instruction::Or(rd, rs1, _rs2) => [0x06, (rd.index() << 4 | rs1.index()) as u8],
            Instruction::Xor(rd, rs1, _rs2) => [0x07, (rd.index() << 4 | rs1.index()) as u8],
            Instruction::Jump(addr) => [0xE0, (addr.value() & 0xFF) as u8],
            Instruction::BranchEq(addr) => [0xE1, (addr.value() & 0xFF) as u8],
            Instruction::Nop => [0xFF, 0x00],
            Instruction::Halt => [0xFF, 0xFF],
        }
    }

    /// Decode instruction from bytes in little-endian format
    pub fn decode(opcode: u8, operand: u8) -> Option<Self> {
        match opcode {
            op => match op & 0xF0 {
                0x80 => Some(Instruction::Load(
                    Register::from_index(op as usize & 0x0F),
                    MemoryAddress::new(0x400 + operand as u16)
                )),
                0x90..=0x9F => Some(Instruction::Store(
                    MemoryAddress::new(0x400 + operand as u16),
                    Register::from_index((opcode & 0x0F) as usize)
                )),
                _ => match opcode {
                    0x03 => {
                        let rd = Register::from_index((operand >> 4) as usize);
                        let rs1 = Register::from_index((operand >> 2 & 0x03) as usize);
                        let rs2 = Register::from_index((operand & 0x03) as usize);
                        Some(Instruction::Add(rd, rs1, rs2))
                    },
                    0x04 => Some(Instruction::Sub(
                        Register::from_index((operand >> 4) as usize),
                        Register::from_index((operand & 0x0F) as usize),
                        Register::from_index(0)
                    )),
                    0x05 => Some(Instruction::And(
                        Register::from_index((operand >> 4) as usize),
                        Register::from_index((operand & 0x0F) as usize),
                        Register::from_index(0)
                    )),
                    0x06 => Some(Instruction::Or(
                        Register::from_index((operand >> 4) as usize),
                        Register::from_index((operand & 0x0F) as usize),
                        Register::from_index(0)
                    )),
                    0x07 => Some(Instruction::Xor(
                        Register::from_index((operand >> 4) as usize),
                        Register::from_index((operand & 0x0F) as usize),
                        Register::from_index(0)
                    )),
                    0xE0 => Some(Instruction::Jump(MemoryAddress::new(operand as u16))),
                    0xE1 => Some(Instruction::BranchEq(MemoryAddress::new(operand as u16))),
                    0xFF => match operand {
                        0x00 => Some(Instruction::Halt),
                        _ => Some(Instruction::Nop),
                    },
                    _ => None,
                }
            }
        }
    }
}

/// Register identifier (0-31)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Register(pub u8);

impl Register {
    pub const R0: Register = Register(0);
    pub const R1: Register = Register(1);
    pub const R2: Register = Register(2);
    pub const R3: Register = Register(3);
    /// Create a new register identifier
    pub fn new(id: u8) -> Option<Self> {
        if id < 32 {
            Some(Register(id))
        } else {
            None
        }
    }
    
    /// Create register from index
    pub fn from_index(idx: usize) -> Self {
        Register((idx & 0x1F) as u8)
    }
    
    /// Get register index
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

/// Memory address
#[derive(Debug, Clone, Copy, PartialEq)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
        Instruction::decode(opcode, operand)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instruction_encoding_decoding() {
        // Test Load instruction
        let load = Instruction::Load(Register::R0, MemoryAddress::new(0x400));
        let encoded = load.encode();
        assert_eq!(encoded[0], 0x80); // opcode with register 0
        assert_eq!(encoded[1], 0x00); // offset
        let decoded = Instruction::decode(encoded[0], encoded[1]);
        assert_eq!(decoded, Some(load));

        // Test Store instruction
        let store = Instruction::Store(MemoryAddress::new(0x402), Register::R1);
        let encoded = store.encode();
        assert_eq!(encoded[0], 0x91); // opcode with register 1
        assert_eq!(encoded[1], 0x02); // offset
        let decoded = Instruction::decode(encoded[0], encoded[1]);
        assert_eq!(decoded, Some(store));

        // Test Add instruction
        let add = Instruction::Add(Register::R1, Register::R0, Register::R1);
        let encoded = add.encode();
        assert_eq!(encoded[0], 0x03); // ADD opcode
        assert_eq!(encoded[1], 0x11); // rd=1(01), rs1=0(00), rs2=1(01)
        let decoded = Instruction::decode(encoded[0], encoded[1]);
        assert_eq!(decoded, Some(add));

        // Test endianness with different register combinations
        let add_r2r3 = Instruction::Add(Register::R2, Register::R3, Register::R1);
        let encoded = add_r2r3.encode();
        assert_eq!(encoded[0], 0x03); // ADD opcode
        assert_eq!(encoded[1], 0x2D); // rd=2(10), rs1=3(11), rs2=1(01)
        let decoded = Instruction::decode(encoded[0], encoded[1]);
        assert_eq!(decoded, Some(add_r2r3));
    }

    #[test]
    fn test_memory_address_encoding() {
        // Test different memory addresses
        let addresses = [0x400, 0x401, 0x402, 0x4FF];
        for addr in addresses {
            let load = Instruction::Load(Register::R0, MemoryAddress::new(addr));
            let encoded = load.encode();
            let decoded = Instruction::decode(encoded[0], encoded[1]);
            assert_eq!(decoded, Some(load), "Failed for address 0x{:X}", addr);
        }
    }

    #[test]
    fn test_register_combinations() {
        // Test all valid register combinations for Add
        for rd in 0..4 {
            for rs1 in 0..4 {
                for rs2 in 0..4 {
                    let add = Instruction::Add(
                        Register::from_index(rd),
                        Register::from_index(rs1),
                        Register::from_index(rs2)
                    );
                    let encoded = add.encode();
                    let decoded = Instruction::decode(encoded[0], encoded[1]);
                    assert_eq!(decoded, Some(add), 
                        "Failed for rd={}, rs1={}, rs2={}", rd, rs1, rs2);
                }
            }
        }
    }
}
