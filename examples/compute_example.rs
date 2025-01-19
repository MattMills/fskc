use fskc::{HolographicKeyPackage, HomomorphicCompute, Operation, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Create encryption system
    let mut rng = ChaCha20Rng::seed_from_u64(12345);
    let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);
    pkg.add_time_step(&mut rng)?;

    // Create compute environment
    let mut compute = HomomorphicCompute::new(pkg);

    // Load values into registers
    let value1 = vec![25u8]; // AX
    let value2 = vec![17u8]; // BX
    println!("Loading values: {} and {}", value1[0], value2[0]);
    
    compute.load(0, &value1)?; // Load into AX
    compute.load(1, &value2)?; // Load into BX

    // Perform XOR operation: AX XOR BX -> AX
    println!("\nPerforming XOR operation (AX XOR BX -> AX)");
    compute.compute(Operation::Xor, 0, 1)?;
    let result = compute.read(0)?;
    println!("XOR result: {} (expected: {})", result[0], value1[0] ^ value2[0]);

    // Reload values for ADD operation
    compute.load(0, &value1)?;
    compute.load(1, &value2)?;

    // Perform ADD operation: AX + BX -> AX
    println!("\nPerforming ADD operation (AX + BX -> AX)");
    compute.compute(Operation::Add, 0, 1)?;
    let result = compute.read(0)?;
    println!("ADD result: {} (expected: {})", result[0], value1[0].wrapping_add(value2[0]));
    println!("Carry flag: {}", (compute.flags() & 1) != 0);

    // Reload values for AND operation
    compute.load(0, &value1)?;
    compute.load(1, &value2)?;

    // Perform AND operation: AX AND BX -> AX
    println!("\nPerforming AND operation (AX AND BX -> AX)");
    compute.compute(Operation::And, 0, 1)?;
    let result = compute.read(0)?;
    println!("AND result: {} (expected: {})", result[0], value1[0] & value2[0]);

    // Reload values for OR operation
    compute.load(0, &value1)?;
    compute.load(1, &value2)?;

    // Perform OR operation: AX OR BX -> AX
    println!("\nPerforming OR operation (AX OR BX -> AX)");
    compute.compute(Operation::Or, 0, 1)?;
    let result = compute.read(0)?;
    println!("OR result: {} (expected: {})", result[0], value1[0] | value2[0]);

    // Test zero flag
    println!("\nTesting zero flag");
    compute.load(0, &vec![0u8])?;
    compute.load(1, &vec![0u8])?;
    compute.compute(Operation::Add, 0, 1)?;
    println!("Zero flag after adding zeros: {}", (compute.flags() & 2) != 0);

    Ok(())
}
