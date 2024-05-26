use criterion::BenchmarkId;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use distrib_kv_store::carp::Carp;

pub fn criterion_benchmark(c: &mut Criterion) {
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
    let mut group = c.benchmark_group("carp::get");

    for i in inputs {
        let len = i.len();
        group.bench_with_input(BenchmarkId::new(format!("{len} nodes"), len), &i, |b, i| {
            b.iter(|| {
                black_box(i.get("0"));
            })
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
