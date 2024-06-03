use std::error::Error;
use distrib_kv_store::cluster_manager::ClusterManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut cluster = ClusterManager::new("Config.toml").await?;

    println!("Application is running. Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await.expect("Failed to listen for ctrl_c");
   
    cluster.shutdown().await?;

    Ok(())
}