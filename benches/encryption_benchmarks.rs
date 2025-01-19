use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fskc::{FractalNode, RovingSelector};

fn bench_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("Encryption");
    
    // Test different data sizes
    for size in [64, 256, 1024, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::new("data_size", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];
                b.iter(|| {
                    black_box(FractalNode::generate(
                        black_box(data.clone()),
                        black_box(12345),
                        black_box(3),
                        black_box(16),
                    ))
                });
            },
        );
    }

    // Test different depths
    let data = vec![0u8; 1024];
    for depth in [1, 2, 3, 4, 5].iter() {
        group.bench_with_input(
            BenchmarkId::new("depth", depth),
            depth,
            |b, &depth| {
                b.iter(|| {
                    black_box(FractalNode::generate(
                        black_box(data.clone()),
                        black_box(12345),
                        black_box(depth),
                        black_box(16),
                    ))
                });
            },
        );
    }

    group.finish();
}

fn bench_roving_selector(c: &mut Criterion) {
    let mut group = c.benchmark_group("RovingSelector");
    
    // Test different dimensions
    for dim in [4, 8, 16, 32].iter() {
        group.bench_with_input(
            BenchmarkId::new("dimension", dim),
            dim,
            |b, &dim| {
                let shared_data = vec![0u8; 100];
                let mut selector = RovingSelector::new(dim, 5, 12345).unwrap();
                selector.map_data(&shared_data).unwrap();
                
                b.iter(|| {
                    black_box(selector.step())
                });
            },
        );
    }

    // Test different particle counts
    for particles in [3, 5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("particles", particles),
            particles,
            |b, &particles| {
                let shared_data = vec![0u8; 100];
                let mut selector = RovingSelector::new(8, particles, 12345).unwrap();
                selector.map_data(&shared_data).unwrap();
                
                b.iter(|| {
                    black_box(selector.step())
                });
            },
        );
    }

    group.finish();
}

fn bench_full_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("FullCycle");
    
    // Test complete encryption-decryption cycle
    for size in [64, 256, 1024].iter() {
        group.bench_with_input(
            BenchmarkId::new("data_size", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];
                let shared_data = vec![0u8; 100];
                
                b.iter(|| {
                    let mut selector = RovingSelector::new(8, 5, 12345).unwrap();
                    selector.map_data(&shared_data).unwrap();
                    
                    let mut combined_seed = 0u64;
                    for _ in 0..5 {
                        let selected = selector.step().unwrap();
                        for &byte in &selected {
                            combined_seed = combined_seed.wrapping_mul(256).wrapping_add(byte as u64);
                        }
                    }
                    
                    let encrypted = FractalNode::generate(
                        data.clone(),
                        combined_seed,
                        3,
                        16,
                    ).unwrap();
                    
                    black_box(encrypted.decrypt())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encryption,
    bench_roving_selector,
    bench_full_cycle
);
criterion_main!(benches);
