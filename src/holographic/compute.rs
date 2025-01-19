use super::{HolographicKeyPackage, Result};

/// Represents a virtual register in the homomorphic computation
#[derive(Clone)]
pub struct Register {
    data: Vec<u8>,
    encrypted: bool,
}

/// Represents available operations in the homomorphic computation
#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Not,
    Shl,
    Shr,
}

impl Operation {
    fn is_reversible(&self) -> bool {
        match self {
            Operation::Add => true,
            Operation::Sub => true,
            Operation::Xor => true,
            Operation::Not => true,
            _ => false
        }
    }
}

/// Provides x86-like computational capabilities over homomorphic encryption
pub struct HomomorphicCompute {
    // The underlying encryption system
    pkg: HolographicKeyPackage,
    // Virtual registers (AX, BX, CX, DX)
    registers: [Register; 4],
    // Flags register (Zero, Sign, Carry, etc.)
    flags: u8,
}

impl Register {
    fn new(pkg: &HolographicKeyPackage, size: usize) -> Result<Self> {
        let mut data = vec![0; size];
        pkg.apply_forward(&mut data)?;
        Ok(Self {
            data,
            encrypted: true,
        })
    }
}

impl HomomorphicCompute {
    /// Creates a new homomorphic computation environment
    pub fn new(pkg: HolographicKeyPackage) -> Result<Self> {
        let registers = [
            Register::new(&pkg, 8)?, // AX
            Register::new(&pkg, 8)?, // BX
            Register::new(&pkg, 8)?, // CX
            Register::new(&pkg, 8)?, // DX
        ];
        Ok(Self {
            pkg,
            registers,
            flags: 0,
        })
    }

    /// Loads a value into a register
    pub fn load(&mut self, reg: usize, value: &[u8]) -> Result<()> {
        if reg >= self.registers.len() {
            return Err(crate::FskcError::EncryptionError("Invalid register".into()));
        }
        
        let mut reg_data = value.to_vec();
        self.pkg.apply_forward(&mut reg_data)?;
        
        self.registers[reg] = Register {
            data: reg_data,
            encrypted: true,
        };
        Ok(())
    }

    /// Performs an operation between two registers, storing result in the first
    pub fn compute(&mut self, op: Operation, dest: usize, src: usize) -> Result<()> {
        if dest >= self.registers.len() || src >= self.registers.len() {
            return Err(crate::FskcError::EncryptionError("Invalid register".into()));
        }

        if !self.registers[dest].encrypted || !self.registers[src].encrypted {
            return Err(crate::FskcError::EncryptionError("Registers must be encrypted".into()));
        }

        // Clone source register data to avoid borrow checker issues
        let src_data = self.registers[src].data.clone();
        
        match op {
            Operation::Add => {
                // Get decrypted values for computation
                let mut dest_val = self.registers[dest].data.clone();
                let mut src_val = src_data.clone();
                self.pkg.apply_backward(&mut dest_val)?;
                self.pkg.apply_backward(&mut src_val)?;

                // Perform addition and handle carry
                let mut result = vec![0u8; dest_val.len()];
                let mut carry = 0u8;
                for i in 0..dest_val.len() {
                    let sum = dest_val[i].wrapping_add(src_val[i]).wrapping_add(carry);
                    carry = if sum < dest_val[i] || (sum == dest_val[i] && carry > 0) { 1 } else { 0 };
                    result[i] = sum;
                }

                // Update carry flag
                self.flags = if carry > 0 { self.flags | 1 } else { self.flags & !1 };

                // Re-encrypt result
                self.pkg.apply_forward(&mut result)?;
                self.registers[dest].data = result;
            },
            Operation::Xor => {
                // XOR is naturally homomorphic
                self.registers[dest].data = self.pkg.homomorphic_operation(
                    &self.registers[dest].data,
                    &src_data
                )?;
            },
            Operation::And => {
                // Get decrypted values for computation
                let mut dest_val = self.registers[dest].data.clone();
                let mut src_val = src_data.clone();
                self.pkg.apply_backward(&mut dest_val)?;
                self.pkg.apply_backward(&mut src_val)?;

                // Perform AND operation
                let mut result = vec![0u8; dest_val.len()];
                for i in 0..dest_val.len() {
                    result[i] = dest_val[i] & src_val[i];
                }

                // Re-encrypt result
                self.pkg.apply_forward(&mut result)?;
                self.registers[dest].data = result;
            },
            Operation::Or => {
                // Get decrypted values for computation
                let mut dest_val = self.registers[dest].data.clone();
                let mut src_val = src_data.clone();
                self.pkg.apply_backward(&mut dest_val)?;
                self.pkg.apply_backward(&mut src_val)?;

                // Perform OR operation
                let mut result = vec![0u8; dest_val.len()];
                for i in 0..dest_val.len() {
                    result[i] = dest_val[i] | src_val[i];
                }

                // Re-encrypt result
                self.pkg.apply_forward(&mut result)?;
                self.registers[dest].data = result;
            },
            Operation::Sub => {
                // Get decrypted values for computation
                let mut dest_val = self.registers[dest].data.clone();
                let mut src_val = src_data.clone();
                self.pkg.apply_backward(&mut dest_val)?;
                self.pkg.apply_backward(&mut src_val)?;

                // Perform subtraction and handle borrow
                let mut result = vec![0u8; dest_val.len()];
                let mut borrow = 0i16;
                for i in 0..dest_val.len() {
                    let a = dest_val[i] as i16;
                    let b = src_val[i] as i16;
                    let diff = a - b - borrow;
                    borrow = if diff < 0 { 1 } else { 0 };
                    result[i] = (diff + (borrow * 256)) as u8;
                }

                // Re-encrypt result
                self.pkg.apply_forward(&mut result)?;
                self.registers[dest].data = result;
            },
            _ => return Err(crate::FskcError::EncryptionError("Operation not implemented".into())),
        }

        // Update zero flag
        let mut result_check = self.registers[dest].data.clone();
        self.pkg.apply_backward(&mut result_check)?;
        if result_check.iter().all(|&x| x == 0) {
            self.flags |= 2;
        } else {
            self.flags &= !2;
        }

        Ok(())
    }

    /// Retrieves the value from a register
    pub fn read(&mut self, reg: usize) -> Result<Vec<u8>> {
        if reg >= self.registers.len() {
            return Err(crate::FskcError::EncryptionError("Invalid register".into()));
        }

        let reg_data = &mut self.registers[reg].data.clone();
        self.pkg.apply_backward(reg_data)?;
        Ok(reg_data.to_vec())
    }

    /// Returns the current flags
    pub fn flags(&self) -> u8 {
        self.flags
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha20Rng;

    fn setup_compute() -> HomomorphicCompute {
        let mut rng = ChaCha20Rng::seed_from_u64(12345);
        let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);
        pkg.add_time_step(&mut rng).unwrap();
        HomomorphicCompute::new(pkg).unwrap()
    }

    #[test]
    fn test_register_operations() -> Result<()> {
        let mut compute = setup_compute();
        
        // Test loading and reading
        let value = vec![25u8];
        compute.load(0, &value)?;
        let result = compute.read(0)?;
        assert_eq!(result, value);

        Ok(())
    }

    #[test]
    fn test_xor_operation() -> Result<()> {
        let mut compute = setup_compute();
        
        let value1 = vec![25u8];
        let value2 = vec![17u8];
        
        compute.load(0, &value1)?;
        compute.load(1, &value2)?;
        
        compute.compute(Operation::Xor, 0, 1)?;
        let result = compute.read(0)?;
        
        assert_eq!(result[0], value1[0] ^ value2[0]);
        Ok(())
    }

    #[test]
    fn test_add_operation() -> Result<()> {
        let mut compute = setup_compute();
        
        let value1 = vec![25u8];
        let value2 = vec![17u8];
        
        compute.load(0, &value1)?;
        compute.load(1, &value2)?;
        
        compute.compute(Operation::Add, 0, 1)?;
        let result = compute.read(0)?;
        
        assert_eq!(result[0], value1[0].wrapping_add(value2[0]));
        Ok(())
    }

    #[test]
    fn test_flags() -> Result<()> {
        let mut compute = setup_compute();
        
        // Test zero flag
        compute.load(0, &vec![0u8])?;
        compute.load(1, &vec![0u8])?;
        compute.compute(Operation::Add, 0, 1)?;
        assert_eq!(compute.flags() & 2, 2, "Zero flag should be set");

        // Test non-zero result
        compute.load(0, &vec![1u8])?;
        compute.load(1, &vec![1u8])?;
        compute.compute(Operation::Add, 0, 1)?;
        assert_eq!(compute.flags() & 2, 0, "Zero flag should not be set");

        // Test carry flag
        compute.load(0, &vec![255u8])?;
        compute.load(1, &vec![1u8])?;
        compute.compute(Operation::Add, 0, 1)?;
        assert_eq!(compute.flags() & 1, 1, "Carry flag should be set");

        Ok(())
    }

    #[test]
    fn test_logical_operations() -> Result<()> {
        let mut compute = setup_compute();
        
        let value1 = vec![0b10101010u8];
        let value2 = vec![0b11001100u8];
        
        // Test AND
        compute.load(0, &value1)?;
        compute.load(1, &value2)?;
        compute.compute(Operation::And, 0, 1)?;
        let result = compute.read(0)?;
        assert_eq!(result[0], value1[0] & value2[0]);

        // Test OR
        compute.load(0, &value1)?;
        compute.load(1, &value2)?;
        compute.compute(Operation::Or, 0, 1)?;
        let result = compute.read(0)?;
        assert_eq!(result[0], value1[0] | value2[0]);

        Ok(())
    }
}
