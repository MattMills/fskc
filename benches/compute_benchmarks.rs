use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fskc::{HolographicKeyPackage, HomomorphicCompute, Operation, Result};
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

fn setup_compute() -> Result<HomomorphicCompute> {
    let mut rng = ChaCha20Rng::seed_from_u64(12345);
    let mut pkg = HolographicKeyPackage::new(vec![0x42; 32]);
    pkg.add_time_step(&mut rng)?;
    HomomorphicCompute::new(pkg)
}

fn bench_operation(c: &mut Criterion, op: Operation, name: &str) {
    let mut compute = setup_compute().unwrap();
    let value1 = vec![25u8];
    let value2 = vec![17u8];
    
    // Load initial values
    compute.load(0, &value1).unwrap();
    compute.load(1, &value2).unwrap();

    c.bench_function(&format!("compute_{}", name), |b| {
        b.iter(|| {
            compute.compute(black_box(op), black_box(0), black_box(1)).unwrap();
        })
    });
}

fn bench_load_operation(c: &mut Criterion) {
    let mut compute = setup_compute().unwrap();
    let value = vec![25u8];

    c.bench_function("compute_load", |b| {
        b.iter(|| {
            compute.load(black_box(0), black_box(&value)).unwrap();
        })
    });
}

fn bench_read_operation(c: &mut Criterion) {
    let mut compute = setup_compute().unwrap();
    compute.load(0, &vec![25u8]).unwrap();

    c.bench_function("compute_read", |b| {
        b.iter(|| {
            black_box(compute.read(black_box(0)).unwrap());
        })
    });
}

fn compute_benchmarks(c: &mut Criterion) {
    // Benchmark each operation type
    bench_operation(c, Operation::Add, "add");
    bench_operation(c, Operation::Xor, "xor");
    bench_operation(c, Operation::And, "and");
    bench_operation(c, Operation::Or, "or");

    // Benchmark register operations
    bench_load_operation(c);
    bench_read_operation(c);

    // Benchmark operation sequences
    let mut compute = setup_compute().unwrap();
    let value1 = vec![25u8];
    let value2 = vec![17u8];
    compute.load(0, &value1).unwrap();
    compute.load(1, &value2).unwrap();

    // Complex sequence: ADD -> XOR -> AND
    c.bench_function("compute_sequence", |b| {
        b.iter(|| {
            compute.compute(Operation::Add, 0, 1).unwrap();
            compute.compute(Operation::Xor, 0, 1).unwrap();
            compute.compute(Operation::And, 0, 1).unwrap();
        })
    });

    // Benchmark with different data sizes
    let sizes = [8, 16, 32, 64, 128];
    for size in sizes {
        let mut compute = setup_compute().unwrap();
        let value1 = vec![0x42; size];
        let value2 = vec![0x17; size];
        compute.load(0, &value1).unwrap();
        compute.load(1, &value2).unwrap();

        c.bench_function(&format!("compute_add_size_{}", size), |b| {
            b.iter(|| {
                compute.compute(Operation::Add, 0, 1).unwrap();
            })
        });
    }
}

criterion_group!(benches, compute_benchmarks);
criterion_main!(benches);
