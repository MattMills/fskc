use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fskc::{
    Result, HolographicKeyPackage, HomomorphicCompute,
    microcontroller::Microcontroller,
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn setup_microcontroller() -> Result<Microcontroller> {
    // Initialize compute engine
    let mut rng = ChaCha20Rng::seed_from_u64(12345);
    let mut key = vec![0u8; 32];
    rng.fill_bytes(&mut key);
    let mut pkg = HolographicKeyPackage::new(key);
    pkg.add_time_step(&mut rng)?;
    let compute = HomomorphicCompute::new(pkg)?;
    
    // Create microcontroller
    Microcontroller::new(compute, 256)
}

fn bench_add_operation(c: &mut Criterion) {
    let mut mc = setup_microcontroller().unwrap();
    
    // Program to add two numbers
    let program = vec![
        0x80, 0x01,  // LD R0, mem[0]
        0x80, 0x12,  // LD R1, mem[1]
        0x03, 0x12,  // ADD R2 <- R0 + R1
        0x82, 0x22,  // ST mem[2], R2
        0xFF, 0x00,  // HALT
    ];
    
    // Test data
    let data1 = vec![42u8; 32];
    let data2 = vec![24u8; 32];
    
    mc.load_program(&program).unwrap();
    mc.load_data(0, &data1).unwrap();
    mc.load_data(1, &data2).unwrap();

    c.bench_function("avr_add", |b| {
        b.iter(|| {
            black_box(mc.execute().unwrap());
        })
    });
}

fn bench_memory_operations(c: &mut Criterion) {
    let mut mc = setup_microcontroller().unwrap();
    let data = vec![42u8; 32];
    
    // Benchmark memory load
    c.bench_function("avr_load", |b| {
        b.iter(|| {
            black_box(mc.load_data(0, &data).unwrap());
        })
    });
    
    // Benchmark memory read
    c.bench_function("avr_read", |b| {
        b.iter(|| {
            black_box(mc.get_memory(0).unwrap());
        })
    });
}

fn bench_program_sizes(c: &mut Criterion) {
    let mut mc = setup_microcontroller().unwrap();
    let data = vec![42u8; 32];
    
    // Test different program sizes
    let sizes = [2, 4, 8, 16, 32]; // Number of instructions
    
    for size in sizes {
        // Create program with given number of ADD instructions
        let mut program = Vec::new();
        for i in 0..size {
            // Load, add, store sequence
            program.extend_from_slice(&[
                0x80, i as u8,     // LD Rx, mem[i]
                0x03, (i+1) as u8, // ADD Rx+1 <- Rx + R0
                0x82, i as u8,     // ST mem[i], Rx
            ]);
        }
        program.extend_from_slice(&[0xFF, 0x00]); // HALT
        
        mc.load_program(&program).unwrap();
        mc.load_data(0, &data).unwrap();
        
        c.bench_function(&format!("avr_program_size_{}", size), |b| {
            b.iter(|| {
                black_box(mc.execute().unwrap());
            })
        });
    }
}

fn bench_conditional_execution(c: &mut Criterion) {
    let mut mc = setup_microcontroller().unwrap();
    
    // Program with conditional branch
    let program = vec![
        0x80, 0x01,  // LD R0, mem[0]
        0x80, 0x12,  // LD R1, mem[1]
        0x03, 0x12,  // ADD R2 <- R0 + R1
        0xF0, 0x02,  // BREQ +2 (skip store if result is zero)
        0x82, 0x22,  // ST mem[2], R2
        0xFF, 0x00,  // HALT
    ];
    
    // Test data that produces zero result
    let zero_case = vec![0u8; 32];
    mc.load_program(&program).unwrap();
    mc.load_data(0, &zero_case).unwrap();
    mc.load_data(1, &zero_case).unwrap();
    
    c.bench_function("avr_branch_taken", |b| {
        b.iter(|| {
            black_box(mc.execute().unwrap());
        })
    });
    
    // Test data that produces non-zero result
    let nonzero_case = vec![42u8; 32];
    mc.load_data(0, &nonzero_case).unwrap();
    
    c.bench_function("avr_branch_not_taken", |b| {
        b.iter(|| {
            black_box(mc.execute().unwrap());
        })
    });
}

fn avr_benchmarks(c: &mut Criterion) {
    bench_add_operation(c);
    bench_memory_operations(c);
    bench_program_sizes(c);
    bench_conditional_execution(c);
}

criterion_group!(benches, avr_benchmarks);
criterion_main!(benches);
