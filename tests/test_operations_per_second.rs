// use distrib_kv_store::cluster_manager::ClusterManager;
use distrib_kv_store::kvclient::KVClient;
use std::error::Error;
use rand::{rngs::StdRng, SeedableRng, Rng, distributions::Alphanumeric};
use std::time::{Duration, Instant};
// use once_cell::sync::Lazy;
// use tokio::sync::Mutex;

// static CLUSTER_MANAGER: Lazy<Mutex<Option<ClusterManager>>> = Lazy::new(|| Mutex::new(None));

// async fn get_or_init_cluster_manager() -> Result<(), Box<dyn Error>> {
//     let mut cluster_manager = CLUSTER_MANAGER.lock().await;
//     if cluster_manager.is_none() {
//         *cluster_manager = Some(ClusterManager::new("Config.toml").await?);
//     }
//     Ok(())
// }

#[tokio::test]
async fn test_write_operations() -> Result<(), Box<dyn Error>> {
    // get_or_init_cluster_manager().await?;

    // tokio::time::sleep(Duration::from_secs(1)).await;

    let mut rng = StdRng::from_entropy(); // StdRng is Send
    let client = KVClient::new("all_nodes.json").await?;

    let mut ops_data: Vec<(f64, usize)> = Vec::new();

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(30);
    let mut i = 0;
    while Instant::now() < end_time {
        let key: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
        let value: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
        client.write(&key, &value).await?;

        if i % 100 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            ops_data.push((elapsed, i));
        }
        i += 1;
    }

    let csv_data = ops_data.iter()
        .map(|&(time, count)| format!("{},{}", time, count))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write("data/ops_data_write.csv", csv_data)?;

    Ok(())
}

#[tokio::test]
async fn test_read_operations() -> Result<(), Box<dyn Error>> {
    // get_or_init_cluster_manager().await?;

    // tokio::time::sleep(Duration::from_secs(1)).await;

    let mut rng = StdRng::from_entropy(); // StdRng is Send
    let client = KVClient::new("all_nodes.json").await?;

    let mut ops_data: Vec<(f64, usize)> = Vec::new();

    let start_time = Instant::now();
    let end_time = start_time + Duration::from_secs(30);
    let mut i = 0;
    while Instant::now() < end_time {
        let key: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
        client.read(&key).await?;

        if i % 100 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            ops_data.push((elapsed, i));
        }
        i += 1;
    }

    let csv_data = ops_data.iter()
        .map(|&(time, count)| format!("{},{}", time, count))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write("data/ops_data_read.csv", csv_data)?;

    Ok(())
}