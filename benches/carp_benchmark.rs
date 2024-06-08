use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use distrib_kv_store::carp::Carp;
use sha2::{Digest, Sha256};

/// Benchmarking the CARP hashing algorithm for different sizes.
pub fn regular_benchmark(c: &mut Criterion) {
    let sizes = [4, 16, 64, 256, 512, 1024, 2048];
    let inputs: Vec<Carp> = sizes
        .iter()
        .map(|&size| {
            Carp::new(
                (0..size)
                    .map(|i| (i.to_string(), 1.0 / size as f32))
                    .collect(),
                0,
            )
        })
        .collect();
    let mut group = c.benchmark_group("CARP Retrieval");

    for i in inputs {
        let size = i.len() as u64;
        group.throughput(Throughput::Elements(size));
        group.bench_with_input(BenchmarkId::from_parameter(size), &i, |b, i| {
            b.iter(|| {
                black_box(i.get("foo"));
            })
        });
    }
}

pub fn hash_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Hashing");
    group.bench_function("CARP", |b| {
        b.iter(|| {
            black_box(membership_hash("Hello, world!"));
        })
    });
    group.bench_function("SHA256", |b| {
        b.iter(|| {
            black_box({
                let mut hasher = Sha256::new();
                hasher.update("Hello, world!".as_bytes());
                let _result = hasher.finalize();
            })
        })
    });
}

/// Only used for the hashing benchmark.
fn membership_hash(addr: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in addr.bytes() {
        let rotated = hash.rotate_left(19).wrapping_add(c as u32);
        hash = hash.wrapping_add(rotated)
    }
    hash = hash.wrapping_add(hash.wrapping_mul(0x62531965));
    hash.rotate_left(21)
}

criterion_group!(benches, regular_benchmark, hash_benchmark);
criterion_main!(benches);
