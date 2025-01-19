use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use fskc::{
    FractalNode, RovingSelector, LayeredCrypto,
    EntropyBuilder, RngEntropy, PhysicalEntropy, EntropySource,
};
use rand::{SeedableRng, RngCore};
use rand_chacha::ChaCha20Rng;

fn bench_entropy(c: &mut Criterion) {
    let mut group = c.benchmark_group("Entropy");
    
    // Test different entropy source combinations
    let data = vec![0u8; 1024];
    let rng1 = ChaCha20Rng::seed_from_u64(12345);
    let rng2 = ChaCha20Rng::seed_from_u64(67890);
    let physical_data = vec![0x42; 1024];
    let stellar_data = vec![0x17; 1024];

    // Single RNG source
    group.bench_function("single_rng", |b| {
        let entropy = EntropyBuilder::new()
            .add_rng(rng1.clone(), "ChaCha20")
            .build();
        let mut output = vec![0u8; 32];
        b.iter(|| {
            let mut guard = entropy.lock().unwrap();
            RngCore::fill_bytes(&mut *guard, &mut output)
        });
    });

    // Multiple RNG sources
    group.bench_function("multiple_rng", |b| {
        let entropy = EntropyBuilder::new()
            .add_rng(rng1.clone(), "ChaCha20 1")
            .add_rng(rng2.clone(), "ChaCha20 2")
            .build();
        let mut output = vec![0u8; 32];
        b.iter(|| {
            let mut guard = entropy.lock().unwrap();
            RngCore::fill_bytes(&mut *guard, &mut output)
        });
    });

    // Physical sources
    group.bench_function("physical_sources", |b| {
        let entropy = EntropyBuilder::new()
            .add_ligo_data(physical_data.clone())
            .add_stellar_parallax(stellar_data.clone())
            .build();
        let mut output = vec![0u8; 32];
        b.iter(|| {
            let mut guard = entropy.lock().unwrap();
            RngCore::fill_bytes(&mut *guard, &mut output)
        });
    });

    // Combined sources
    group.bench_function("combined_sources", |b| {
        let entropy = EntropyBuilder::new()
            .add_rng(rng1.clone(), "ChaCha20")
            .add_ligo_data(physical_data.clone())
            .add_stellar_parallax(stellar_data.clone())
            .build();
        let mut output = vec![0u8; 32];
        b.iter(|| {
            let mut guard = entropy.lock().unwrap();
            RngCore::fill_bytes(&mut *guard, &mut output)
        });
    });

    group.finish();
}

fn bench_layered_crypto(c: &mut Criterion) {
    let mut group = c.benchmark_group("LayeredCrypto");
    
    // Test different data sizes
    for size in [64, 256, 1024, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::new("data_size", size),
            size,
            |b, &size| {
                let data = vec![0u8; size];
                let mut crypto = LayeredCrypto::new(67890);
                b.iter(|| {
                    black_box(crypto.encrypt(
                        black_box(&data),
                        black_box(12345),
                    ))
                });
            },
        );
    }

    // Compare basic vs layered for medium size data
    let data = vec![0u8; 1024];
    group.bench_function("basic_vs_layered/basic", |b| {
        b.iter(|| {
            black_box(FractalNode::generate(
                black_box(data.clone()),
                black_box(12345),
                black_box(3),
                black_box(64),
            ))
        });
    });

    group.bench_function("basic_vs_layered/layered", |b| {
        let mut crypto = LayeredCrypto::new(67890);
        b.iter(|| {
            black_box(crypto.encrypt(
                black_box(&data),
                black_box(12345),
            ))
        });
    });

    // Full cycle (encrypt + decrypt)
    group.bench_function("full_cycle", |b| {
        let data = vec![0u8; 1024];
        let mut crypto = LayeredCrypto::new(67890);
        b.iter(|| {
            let encrypted = crypto.encrypt(
                black_box(&data),
                black_box(12345),
            ).unwrap();
            black_box(crypto.decrypt(
                black_box(&encrypted),
                black_box(12345),
            ))
        });
    });

    group.finish();
}

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
    bench_full_cycle,
    bench_layered_crypto,
    bench_entropy
);
criterion_main!(benches);
