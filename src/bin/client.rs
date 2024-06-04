use distrib_kv_store::kvclient::KVClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = KVClient::new("all_nodes.json").await?;

    client.write("key", "value").await?;
    client.write("hi", "test").await?;
    client.write("hello", "testing").await?;

    match client.read("hello").await {
        Ok(response) => println!("Value retrieved: {}", response),
        Err(e) => println!("Error reading value: {}", e),
    }

    println!("{:?}", client.carp_ring.proxy_map);

    Ok(())
}