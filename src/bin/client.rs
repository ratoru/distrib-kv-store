use distrib_kv_store::raft_node::RaftNode;
use distrib_kv_store::store::Request;
use distrib_kv_store::carp::Carp;

use std::collections::HashMap;
use rand::seq::IteratorRandom;

use std::fs;
use std::error::Error;

use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let data = fs::read_to_string("all_nodes.json")?;
    let all_nodes: Vec<Vec<String>> = serde_json::from_str(&data)?;

    let mut node_map = HashMap::new();
    for (_, nodes) in all_nodes.iter().enumerate() {
        let leader = RaftNode::new(1, nodes[0].clone());
        node_map.insert(nodes[0].clone(), leader);
    }

    let carp_ring: Carp;

    // Select a random node from the node_map
    let mut rng = rand::thread_rng();
    if let Some((_, random_node)) = node_map.iter().choose(&mut rng) {
        carp_ring = random_node.get_hash_ring().await?;
    } else {
        println!("No nodes available in the node_map to get the Carp ring.");
        return Ok(());
    }

    write(&carp_ring, &node_map, "key", "value").await?;
    write(&carp_ring, &node_map, "hi", "test").await?;
    write(&carp_ring, &node_map, "hello", "testing").await?;

    match read(&carp_ring, &node_map, "hello").await {
        Ok(response) => println!("{}", response),
        Err(e) => println!("Error reading value: {}", e),
    }

    Ok(())
}

async fn write(
    carp_ring: &Carp,
    node_map: &HashMap<String, RaftNode>,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let responsible_node_addr = carp_ring.get(key);
    if let Some(responsible_node) = node_map.get(responsible_node_addr) {
        responsible_node.write(&Request::Set {
            key: key.to_string(),
            value: value.to_string(),
        }).await?;
    } else {
        println!("No RaftNode found for the address: {}", responsible_node_addr);
    }
    Ok(())
}

async fn read(
    carp_ring: &Carp,
    node_map: &HashMap<String, RaftNode>,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let responsible_node_addr = carp_ring.get(key);

    if let Some(responsible_node) = node_map.get(responsible_node_addr) {
        let response = responsible_node.read(&key.to_string()).await?;
        Ok(response)
    } else {
        println!("No RaftNode found for the address: {}", responsible_node_addr);
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "RaftNode not found")))
    }
}