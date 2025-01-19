use fskc::{
    Result,
    HolographicKeyPackage,
    HomomorphicCompute,
    zkp_container::ZkpContainer,
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn main() -> Result<()> {
    // Initialize compute engine
    let mut rng = ChaCha20Rng::seed_from_u64(12345);
    let mut key = vec![0u8; 32];
    rng.fill_bytes(&mut key);
    let mut pkg = HolographicKeyPackage::new(key);
    pkg.add_time_step(&mut rng)?;
    let compute = HomomorphicCompute::new(pkg)?;

    // Create a sequence of states
    let states = vec![
        vec![10u8; 32], // Initial state
        vec![20u8; 32], // Step 1
        vec![30u8; 32], // Step 2
        vec![40u8; 32], // Step 3
    ];

    println!("Creating ZKP container with initial state...");
    let mut container = ZkpContainer::new(compute, states[0].clone())?;
    println!("Current state: {:?}\n", container.state()[0]);

    // Forward progression with proofs
    println!("Forward progression:");
    let mut forward_proofs = Vec::new();
    for i in 1..states.len() {
        println!("Generating proof for state {} -> {}", i-1, i);
        let proof = container.prove_next(&states[i])?;
        forward_proofs.push(proof.clone());

        println!("Verifying and advancing...");
        container.advance(states[i].clone(), &proof)?;
        println!("Current state: {:?}\n", container.state()[0]);
    }

    // Backward progression with proofs
    println!("Backward progression:");
    let mut backward_proofs = Vec::new();
    for i in (0..states.len()-1).rev() {
        println!("Generating proof for state {} -> {}", i+1, i);
        let proof = container.prove_previous(&states[i])?;
        backward_proofs.push(proof.clone());

        println!("Verifying and reversing...");
        container.reverse(states[i].clone(), &proof)?;
        println!("Current state: {:?}\n", container.state()[0]);
    }

    // Demonstrate invalid transitions
    println!("Testing invalid transitions:");
    
    // Try to advance with wrong state/proof combination
    println!("Attempting advance with mismatched state/proof...");
    let invalid_state = vec![99u8; 32];
    let result = container.advance(invalid_state.clone(), &forward_proofs[0]);
    match result {
        Ok(_) => println!("Error: Invalid transition succeeded when it should fail"),
        Err(e) => println!("Success: Invalid transition failed as expected: {}", e),
    }
    println!();

    // Try to reverse with wrong state/proof combination
    println!("Attempting reverse with mismatched state/proof...");
    let result = container.reverse(invalid_state, &backward_proofs[0]);
    match result {
        Ok(_) => println!("Error: Invalid transition succeeded when it should fail"),
        Err(e) => println!("Success: Invalid transition failed as expected: {}", e),
    }

    Ok(())
}
