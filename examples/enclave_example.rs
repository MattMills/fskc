use fskc::{
    Result, HolographicKeyPackage, HomomorphicCompute,
    microcontroller::Microcontroller,
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Initialize root key package (this key is protected)
    let mut rng = ChaCha20Rng::seed_from_u64(12345);
    let mut root_key = vec![0u8; 32];
    rng.fill_bytes(&mut root_key);
    let mut root_pkg = HolographicKeyPackage::new(root_key);
    root_pkg.add_time_step(&mut rng)?;

    // Create homomorphic compute engine
    // This will automatically derive a key for the execution environment
    let compute = HomomorphicCompute::new(root_pkg)?;

    // Create AVR microcontroller
    let mut mc = Microcontroller::new(compute, 1024)?; // 1KB memory

    println!("Loading program into memory...");
    // Simple program to add two numbers homomorphically
    let program = vec![
        0x80, 0x00,  // LD R0, mem[0x400]    ; Load first operand into R0
        0x81, 0x01,  // LD R1, mem[0x401]    ; Load second operand into R1
        0x03, 0x20,  // ADD R2 <- R0 + R1    ; Add R0 and R1, store in R2
        0x90, 0x02,  // ST mem[0x402], R2    ; Store R2 to memory
        0xFF, 0x00,  // HALT                 ; Stop execution
    ];

    // Load program into memory
    mc.load_program(&program)?;

    // Load operands (these will be encrypted with the derived key)
    let operand1 = vec![42u8; 32];
    let operand2 = vec![17u8; 32];
    mc.load_data(0x400, &operand1)?;
    mc.load_data(0x401, &operand2)?;

    println!("\nExecuting program in homomorphic space...");
    mc.execute()?;

    // Read result
    println!("\nReading result...");
    let result = mc.get_memory(0x402)?;
    println!("Result: {:?}", result);

    // Verify result matches expected
    assert_eq!(result[0], operand1[0].wrapping_add(operand2[0]));
    println!("\nProgram executed successfully!");

    Ok(())
}
