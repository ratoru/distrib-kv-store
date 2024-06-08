use distrib_kv_store::kvclient::KVClient;
use std::error::Error;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = KVClient::new("all_nodes.json").await?;
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    println!("Welcome to the KV Store REPL. Type 'write <key> <value>' to write and 'read <key>' to read.");
    eprint!("kvstore> ");

    while let Ok(Some(line)) = lines.next_line().await {
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "write" if parts.len() == 3 => {
                let key = parts[1];
                let value = parts[2];
                match client.write(key, value).await {
                    Ok(_) => println!("Successfully wrote {}: {}", key, value),
                    Err(e) => println!("Error writing value: {}", e),
                }
            }
            "read" if parts.len() == 2 => {
                let key = parts[1];
                match client.read(key).await {
                    Ok(response) => println!("Value retrieved: {}", response),
                    Err(e) => println!("Error reading value: {}", e),
                }
            }
            "consistent_read" if parts.len() == 2 => {
                let key = parts[1];
                match client.consistent_read(key).await {
                    Ok(response) => println!("Value retrieved: {}", response),
                    Err(e) => println!("Error reading value: {}", e),
                }
            }
            _ => println!("Invalid command. Use 'write <key> <value>' or 'read <key>' or 'consistent_read <key>'."),
        }
        eprint!("kvstore> ");
    }

    Ok(())
}