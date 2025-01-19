use crate::{
    Result, FskcError,
    HomomorphicCompute, Operation,
    microcontroller::{Instruction, Register, MemoryAddress, StatusFlags},
};

/// Register state
#[derive(Debug)]
pub struct RegisterState {
    /// General purpose registers
    registers: Vec<Vec<u8>>,
    /// Status flags
    flags: StatusFlags,
}

impl RegisterState {
    /// Create new register state
    pub fn new(num_registers: usize) -> Self {
        Self {
            registers: vec![vec![0u8; 32]; num_registers],
            flags: StatusFlags::new(),
        }
    }

    /// Read register value
    pub fn read(&self, reg: Register) -> Result<&[u8]> {
        self.registers.get(reg.index())
            .map(|v| v.as_slice())
            .ok_or_else(|| FskcError::Custom("Invalid register".into()))
    }

    /// Write register value
    pub fn write(&mut self, reg: Register, data: &[u8]) -> Result<()> {
        if let Some(slot) = self.registers.get_mut(reg.index()) {
            if data.len() != slot.len() {
                return Err(FskcError::Custom("Invalid data size".into()));
            }
            slot.copy_from_slice(data);
            Ok(())
        } else {
            Err(FskcError::Custom("Invalid register".into()))
        }
    }

    /// Get status flags
    pub fn flags(&self) -> StatusFlags {
        self.flags
    }

    /// Set status flags
    pub fn set_flags(&mut self, flags: StatusFlags) {
        self.flags = flags;
    }
}

/// Execution mode for the enclave
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    /// Normal execution mode - operations in plaintext
    Clear,
    /// Blocked execution mode - operations are homomorphic
    Blocked,
}

/// Memory protection level
#[derive(Debug, Clone, PartialEq)]
pub enum ProtectionLevel {
    /// Memory accessible in any mode
    Unprotected,
    /// Memory only accessible in blocked mode
    Protected,
    /// Memory only accessible for specific operations
    Restricted(Vec<Instruction>),
}

/// Protected memory region
#[derive(Debug)]
pub struct MemoryRegion {
    /// Start address of region
    pub start: usize,
    /// End address of region
    pub end: usize,
    /// Protection level
    pub protection: ProtectionLevel,
    /// Encrypted memory contents
    data: Vec<Vec<u8>>,
}

impl MemoryRegion {
    /// Create new memory region
    pub fn new(start: usize, end: usize, protection: ProtectionLevel) -> Self {
        let size = end - start;
        Self {
            start,
            end,
            protection,
            data: vec![vec![0u8; 32]; size],
        }
    }

    /// Check if address is in region
    pub fn contains(&self, addr: usize) -> bool {
        addr >= self.start && addr < self.end
    }

    /// Read data at offset
    pub fn read(&self, offset: usize) -> Result<&[u8]> {
        self.data.get(offset)
            .map(|v| v.as_slice())
            .ok_or_else(|| FskcError::Custom("Invalid memory access".into()))
    }

    /// Write data at offset
    pub fn write(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        if let Some(slot) = self.data.get_mut(offset) {
            if data.len() != slot.len() {
                return Err(FskcError::Custom("Invalid data size".into()));
            }
            slot.copy_from_slice(data);
            Ok(())
        } else {
            Err(FskcError::Custom("Invalid memory access".into()))
        }
    }
}

/// Block execution context
#[derive(Debug)]
pub struct BlockContext {
    /// Current execution mode
    pub mode: ExecutionMode,
    /// Program counter
    pub pc: usize,
    /// Block start address
    pub start_addr: usize,
    /// Block end address
    pub end_addr: usize,
    /// Protected memory regions
    pub regions: Vec<MemoryRegion>,
    /// Compute engine for homomorphic operations
    compute: HomomorphicCompute,
    registers: RegisterState,
}

impl BlockContext {
    /// Create new block context
    /// Create new block context
    pub fn new(compute: HomomorphicCompute) -> Self {
        Self {
            mode: ExecutionMode::Clear,
            pc: 0,
            start_addr: 0,
            end_addr: 0,
            regions: Vec::new(),
            compute,
            registers: RegisterState::new(16), // 16 general purpose registers
        }
    }

    /// Enter blocked mode
    pub fn enter_blocked_mode(&mut self, start: usize, end: usize) -> Result<()> {
        self.mode = ExecutionMode::Blocked;
        self.start_addr = start;
        self.end_addr = end;
        Ok(())
    }

    /// Exit blocked mode
    pub fn exit_blocked_mode(&mut self) -> Result<()> {
        self.mode = ExecutionMode::Clear;
        self.start_addr = 0;
        self.end_addr = 0;
        Ok(())
    }

    /// Add protected memory region
    pub fn add_region(&mut self, region: MemoryRegion) {
        self.regions.push(region);
    }

    /// Find region containing address
    pub fn find_region(&self, addr: usize) -> Option<&MemoryRegion> {
        self.regions.iter().find(|r| r.contains(addr))
    }

    /// Find mutable region containing address
    pub fn find_region_mut(&mut self, addr: usize) -> Option<&mut MemoryRegion> {
        self.regions.iter_mut().find(|r| r.contains(addr))
    }

    /// Check if operation is allowed in current mode
    pub fn check_operation(&self, instr: &Instruction) -> bool {
        match self.mode {
            // In clear mode, only allow operations on unprotected memory
            ExecutionMode::Clear => {
                if let Some(addr) = self.get_instruction_address(instr) {
                    if let Some(region) = self.find_region(addr) {
                        match region.protection {
                            ProtectionLevel::Unprotected => true,
                            _ => false
                        }
                    } else {
                        true // Unprotected memory
                    }
                } else {
                    false // Non-memory operations not allowed in clear mode
                }
            },
            // In blocked mode, allow all operations
            ExecutionMode::Blocked => true
        }
    }

    /// Check if memory access is allowed
    pub fn check_access(&self, addr: usize, instr: Option<Instruction>) -> bool {
        if let Some(region) = self.find_region(addr) {
            match region.protection {
                ProtectionLevel::Unprotected => true,
                ProtectionLevel::Protected => self.mode == ExecutionMode::Blocked,
                ProtectionLevel::Restricted(ref allowed) => {
                    self.mode == ExecutionMode::Blocked && 
                    instr.map_or(false, |i| allowed.contains(&i))
                }
            }
        } else {
            true // Unprotected memory
        }
    }

    /// Read memory
    pub fn read_memory(&self, addr: usize) -> Result<&[u8]> {
        if !self.check_access(addr, None) {
            return Err(FskcError::Custom("Memory access denied".into()));
        }

        if let Some(region) = self.find_region(addr) {
            let offset = addr - region.start;
            region.read(offset)
        } else {
            Err(FskcError::Custom("Invalid memory access".into()))
        }
    }

    /// Write memory
    pub fn write_memory(&mut self, addr: usize, data: &[u8]) -> Result<()> {
        if !self.check_access(addr, None) {
            return Err(FskcError::Custom("Memory access denied".into()));
        }

        if let Some(region) = self.find_region_mut(addr) {
            let offset = addr - region.start;
            region.write(offset, data)
        } else {
            Err(FskcError::Custom("Invalid memory access".into()))
        }
    }

    /// Execute instruction in current mode
    pub fn execute(&mut self, instr: Instruction) -> Result<()> {
        // Check if operation is allowed in current mode
        if !self.check_operation(&instr) {
            return Err(FskcError::Custom("Operation not allowed in current mode".into()));
        }

        match self.mode {
            ExecutionMode::Clear => self.execute_clear(instr),
            ExecutionMode::Blocked => self.execute_blocked(instr),
        }
    }

    fn get_instruction_address(&self, instr: &Instruction) -> Option<usize> {
        match instr {
            Instruction::Load(_, addr) => Some(addr.value()),
            Instruction::Store(addr, _) => Some(addr.value()),
            _ => None,
        }
    }

    /// Execute instruction in clear mode
    fn execute_clear(&mut self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::Load(rd, addr) => {
                let data = self.read_memory(addr.value())?.to_vec();
                self.registers.write(rd, &data)
            }
            Instruction::Store(addr, rs) => {
                let data = self.registers.read(rs)?.to_vec();
                self.write_memory(addr.value(), &data)
            }
            Instruction::Add(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                let mut result = vec![0u8; op1.len()];
                let mut carry = false;
                
                for i in 0..op1.len() {
                    let sum = op1[i] as u16 + op2[i] as u16 + if carry { 1 } else { 0 };
                    result[i] = sum as u8;
                    carry = sum > 255;
                }
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::Xor(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                let mut result = vec![0u8; op1.len()];
                for i in 0..op1.len() {
                    result[i] = op1[i] ^ op2[i];
                }
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::And(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                let mut result = vec![0u8; op1.len()];
                for i in 0..op1.len() {
                    result[i] = op1[i] & op2[i];
                }
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::Or(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                let mut result = vec![0u8; op1.len()];
                for i in 0..op1.len() {
                    result[i] = op1[i] | op2[i];
                }
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::Jump(addr) => {
                self.pc = addr.value() as usize;
                Ok(())
            }
            Instruction::BranchEq(addr) => {
                if self.registers.flags().zero {
                    self.pc = addr.value() as usize;
                }
                Ok(())
            }
            _ => Ok(())
        }
    }

    /// Execute instruction in blocked mode
    fn execute_blocked(&mut self, instr: Instruction) -> Result<()> {
        match instr {
            Instruction::Load(rd, addr) => {
                let data = self.read_memory(addr.value())?.to_vec();
                self.registers.write(rd, &data)
            }
            Instruction::Store(addr, rs) => {
                let data = self.registers.read(rs)?.to_vec();
                self.write_memory(addr.value(), &data)
            }
            Instruction::Add(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                self.compute.load(0, op1)?;
                self.compute.load(1, op2)?;
                self.compute.compute(Operation::Add, 0, 1)?;
                let result = self.compute.read(0)?;
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::Xor(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                self.compute.load(0, op1)?;
                self.compute.load(1, op2)?;
                self.compute.compute(Operation::Xor, 0, 1)?;
                let result = self.compute.read(0)?;
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::And(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                self.compute.load(0, op1)?;
                self.compute.load(1, op2)?;
                self.compute.compute(Operation::And, 0, 1)?;
                let result = self.compute.read(0)?;
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::Or(rd, rs1, rs2) => {
                let op1 = self.registers.read(rs1)?;
                let op2 = self.registers.read(rs2)?;
                
                self.compute.load(0, op1)?;
                self.compute.load(1, op2)?;
                self.compute.compute(Operation::Or, 0, 1)?;
                let result = self.compute.read(0)?;
                
                let mut flags = self.registers.flags();
                flags.zero = result.iter().all(|&x| x == 0);
                flags.negative = result[0] & 0x80 != 0;
                self.registers.set_flags(flags);
                
                self.registers.write(rd, &result)
            }
            Instruction::Jump(addr) => {
                let addr_val = addr.value() as usize;
                if addr_val >= self.start_addr && addr_val < self.end_addr {
                    self.pc = addr.value() as usize;
                    Ok(())
                } else {
                    Err(FskcError::Custom("Jump outside block".into()))
                }
            }
            Instruction::BranchEq(addr) => {
                if self.registers.flags().zero {
                    let addr_val = addr.value() as usize;
                    if addr_val >= self.start_addr && addr_val < self.end_addr {
                        self.pc = addr.value() as usize;
                        Ok(())
                    } else {
                        Err(FskcError::Custom("Branch outside block".into()))
                    }
                } else {
                    Ok(())
                }
            }
            _ => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HolographicKeyPackage;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::ChaCha20Rng;

    fn setup_context() -> Result<BlockContext> {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let mut key = vec![0u8; 32];
        rng.fill_bytes(&mut key);
        let mut pkg = HolographicKeyPackage::new(key);
        pkg.add_time_step(&mut rng)?;
        let compute = HomomorphicCompute::new(pkg)?;
        Ok(BlockContext::new(compute))
    }

    #[test]
    fn test_memory_protection() -> Result<()> {
        let mut ctx = setup_context()?;

        // Add protected region
        let region = MemoryRegion::new(
            0x1000,
            0x2000,
            ProtectionLevel::Protected
        );
        ctx.add_region(region);

        // Access should be denied in clear mode
        assert!(!ctx.check_access(0x1500, None));

        // Enter blocked mode
        ctx.enter_blocked_mode(0, 100)?;

        // Access should be allowed in blocked mode
        assert!(ctx.check_access(0x1500, None));

        Ok(())
    }

    #[test]
    fn test_restricted_access() -> Result<()> {
        let mut ctx = setup_context()?;

        // Add restricted region
        let region = MemoryRegion::new(
            0x1000,
            0x2000,
            ProtectionLevel::Restricted(vec![
                Instruction::Load(Register::R0, MemoryAddress::new(0))
            ])
        );
        ctx.add_region(region);

        // Enter blocked mode
        ctx.enter_blocked_mode(0, 100)?;

        // Only allowed instruction should work
        assert!(ctx.check_access(0x1500, Some(
            Instruction::Load(Register::R0, MemoryAddress::new(0))
        )));

        assert!(!ctx.check_access(0x1500, Some(
            Instruction::Store(MemoryAddress::new(0), Register::R0)
        )));

        Ok(())
    }
}
