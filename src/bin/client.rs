use std::time::Duration;

use distrib_kv_store::raft_node::RaftNode;
use distrib_kv_store::start_example_raft_node;
use distrib_kv_store::store::Request;

use distrib_kv_store::carp::Carp;
use std::collections::HashMap;

use std::fs;
use std::error::Error;

use toml;
use serde::Deserialize;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // work in progress

    Ok(())
}


async fn write_to_responsible_node(
    carp_ring: &Carp,
    node_map: &HashMap<String, RaftNode>,
    key: &str,
    value: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let responsible_node_addr = carp_ring.get(key);
    println!("Key: '{}', Value: '{}', Responsible Node: {}", key, value, responsible_node_addr);

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

async fn read_from_responsible_node(
    carp_ring: &Carp,
    node_map: &HashMap<String, RaftNode>,
    key: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let responsible_node_addr = carp_ring.get(key);
    println!("Key: '{}', Responsible Node: {}", key, responsible_node_addr);

    if let Some(responsible_node) = node_map.get(responsible_node_addr) {
        let response = responsible_node.read(&key.to_string()).await?;
        Ok(response)
    } else {
        println!("No RaftNode found for the address: {}", responsible_node_addr);
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "RaftNode not found")))
    }
}