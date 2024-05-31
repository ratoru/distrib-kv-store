use distrib_kv_store::kvclient::KVClient;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = KVClient::new().await?;

    client.write("key", "value").await?;
    client.write("hi", "test").await?;
    client.write("hello", "testing").await?;

    match client.read("hello").await {
        Ok(response) => println!("Value retrieved: {}", response),
        Err(e) => println!("Error reading value: {}", e),
    }

    Ok(())
}