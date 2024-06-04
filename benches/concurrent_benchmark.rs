use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use tokio::runtime::Runtime;
use rand::{distributions::Alphanumeric, Rng};
use distrib_kv_store::kvclient::KVClient;

fn benchmark_read(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut rng = rand::thread_rng();

    let mut group = c.benchmark_group("read_scaling");
    for num_clients in [1, 2, 5, 10].iter() {
        group.throughput(Throughput::Elements(*num_clients as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_clients), num_clients, |b, &num_clients| {

            let clients: Vec<KVClient> = (0..num_clients).map(|_| {
                rt.block_on(KVClient::new("all_nodes.json")).unwrap()
            }).collect();

            b.iter(|| {
                let futures: Vec<_> = clients.iter().map(|client| {
                    let key: String = (0..10)
                        .map(|_| rng.sample(Alphanumeric) as char)
                        .collect();
                    async move {
                        client.read(&key).await
                    }
                }).collect();

                rt.block_on(async {
                    futures::future::join_all(futures).await;
                });
            });
        });
    }
    group.finish();
}

fn benchmark_write(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut rng = rand::thread_rng();

    let mut group = c.benchmark_group("write_scaling");
    for num_clients in [1, 2, 5, 10].iter() {
        group.throughput(Throughput::Elements(*num_clients as u64));
        group.bench_with_input(BenchmarkId::from_parameter(num_clients), num_clients, |b, &num_clients| {

            let clients: Vec<KVClient> = (0..num_clients).map(|_| {
                rt.block_on(KVClient::new("all_nodes.json")).unwrap()
            }).collect();

            b.iter(|| {
                let futures: Vec<_> = clients.iter().map(|client| {
                    let key: String = (0..10)
                        .map(|_| rng.sample(Alphanumeric) as char)
                        .collect();
                    let value: String = (0..10)
                        .map(|_| rng.sample(Alphanumeric) as char)
                        .collect();
                    
                    async move {
                        client.write(&key, &value).await
                    }
                }).collect();

                rt.block_on(async {
                    futures::future::join_all(futures).await;
                });
            });
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_read, benchmark_write);
criterion_main!(benches);