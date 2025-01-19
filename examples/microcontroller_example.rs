use fskc::{
    Result,
    HolographicKeyPackage,
    HomomorphicCompute,
    microcontroller::Microcontroller,
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Initialize RNG and key package
    let mut rng = ChaCha20Rng::seed_from_u64(12345);
    let mut key = vec![0u8; 32];
    rng.fill_bytes(&mut key);
    
    let mut pkg = HolographicKeyPackage::new(key);
    pkg.add_time_step(&mut rng)?;
    
    // Create microcontroller with compute engine
    let mut mc = Microcontroller::new(HomomorphicCompute::new(pkg)?, 1024)?;
    
    // Example program:
    // 1. Load numbers from memory into registers
    // 2. Add them together
    // 3. Store result back to memory
    let program = vec![
        // Load numbers from memory
        0x80, 0x01,  // LD R0, mem[0]  (R0 = first number)
        0x80, 0x12,  // LD R1, mem[1]  (R1 = second number)
        
        // Add numbers and store result
        0x03, 0x12,  // ADD R2 <- R0 + R1
        0x82, 0x22,  // ST mem[2], R2
        
        // Done
        0xFF, 0x00,  // HALT
    ];
    
    // Load program and initial data
    mc.load_program(&program)?;
    
    // Load test values into memory
    let data1 = vec![42u8; 32];  // First number: 42
    let data2 = vec![24u8; 32];  // Second number: 24
    
    println!("Loading initial data...");
    mc.load_data(0, &data1)?;
    mc.load_data(1, &data2)?;
    
    // Show initial state
    println!("\nInitial state:");
    println!("Memory[0] = {:?} (42)", mc.get_memory(0)?);
    println!("Memory[1] = {:?} (24)", mc.get_memory(1)?);
    
    // Execute program
    println!("\nExecuting program...");
    mc.execute()?;
    
    // Show result
    println!("\nFinal state:");
    println!("Memory[2] = {:?} (42 + 24 = 66)", mc.get_memory(2)?);
    
    Ok(())
}
