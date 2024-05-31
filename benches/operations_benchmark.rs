use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;
use rand::{distributions::Alphanumeric, Rng};
use distrib_kv_store::kvclient::KVClient;

fn benchmark_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = rt.block_on(KVClient::new()).unwrap();
    let mut rng = rand::thread_rng();

    c.bench_function("read", |b| {
        b.iter(|| {
            let key: String = (0..10)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();

            rt.block_on(client.read(&key)).unwrap();
        });
    });
}

fn benchmark_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = rt.block_on(KVClient::new()).unwrap();
    let mut rng = rand::thread_rng();

    c.bench_function("write", |b| {
        b.iter(|| {
            let key: String = (0..10)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            let value: String = (0..10)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();

            rt.block_on(client.write(&key, &value)).unwrap();
        });
    });
}

fn benchmark_mixed_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = rt.block_on(KVClient::new()).unwrap();
    let mut rng = rand::thread_rng();

    c.bench_function("mixed_read_write", |b| {
        b.iter(|| {
            let key: String = (0..10)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            let value: String = (0..10)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            let operation = rng.gen_range(0..2);


            if operation == 0 {
                rt.block_on(client.read(&key)).unwrap();
            } else {
                rt.block_on(client.write(&key, &value)).unwrap();
            }
        });
    });
}

criterion_group!(benches, benchmark_read, benchmark_write, benchmark_mixed_operations);
criterion_main!(benches);