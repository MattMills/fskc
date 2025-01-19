use fskc::{
    Result,
    HolographicKeyPackage,
    HomomorphicCompute,
    vm::VirtualMachine,
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
    
    // Create VM with compute engine
    let mut vm = VirtualMachine::new(HomomorphicCompute::new(pkg)?, 256)?;
    
    // Example program: Add two numbers and store result
    let program = vec![
        // Instruction format: [opcode, param1, param2, param3, padding...]
        {
            let mut instr = vec![0u8; 32];
            instr[0] = 0;  // Load
            instr[1] = 0;  // src = memory[0]
            instr[2] = 1;  // dst = register[1]
            instr
        },
        {
            let mut instr = vec![0u8; 32];
            instr[0] = 0;  // Load
            instr[1] = 1;  // src = memory[1]
            instr[2] = 2;  // dst = register[2]
            instr
        },
        {
            let mut instr = vec![0u8; 32];
            instr[0] = 3;  // Add
            instr[1] = 1;  // src1 = register[1]
            instr[2] = 2;  // src2 = register[2]
            instr[3] = 3;  // dst = register[3]
            instr
        },
        {
            let mut instr = vec![0u8; 32];
            instr[0] = 1;  // Store
            instr[1] = 3;  // src = register[3]
            instr[2] = 2;  // dst = memory[2]
            instr
        },
        {
            let mut instr = vec![0u8; 32];
            instr[0] = 8;  // Halt
            instr
        },
    ];
    
    // Load initial data into memory
    let data1 = vec![42u8; 32];  // First number
    let data2 = vec![24u8; 32];  // Second number
    
    vm.load_program(&program)?;
    
    // Load initial data into memory
    vm.load_data(0, &data1)?;  // First number
    vm.load_data(1, &data2)?;  // Second number
    
    // Execute program
    println!("Initial state:");
    println!("Memory[0]: {:?} (normalized from 42)", vm.get_memory(0));
    println!("Memory[1]: {:?} (normalized from 24)", vm.get_memory(1));
    
    vm.execute()?;
    
    println!("\nFinal state:");
    println!("Memory[2] (result): {:?} (42 + 24 = 66)", vm.get_memory(2));
    
    Ok(())
}
