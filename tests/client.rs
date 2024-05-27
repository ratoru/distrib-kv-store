// use std::backtrace::Backtrace;
// use std::collections::BTreeMap;
// use std::panic::PanicInfo;
use std::thread;
use std::time::Duration;

use distrib_kv_store::node::RaftNode;
use distrib_kv_store::start_example_raft_node;
use distrib_kv_store::store::Request;
// use distrib_kv_store::Node;
// use maplit::btreemap;
// use maplit::btreeset;
use tokio::runtime::Handle;
// use tracing_subscriber::EnvFilter;

use distrib_kv_store::carp::Carp;
use std::collections::HashMap;
// use tokio::sync::RwLock;
// use std::sync::Arc;

/// Setup multiple Raft clusters in a CARP ring.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    fn get_addr(node_id: u64, cluster_id: u64) -> String {
        format!("127.0.0.1:{}", 31000 + cluster_id * 100 + node_id)
    }
    
    fn get_rpc_addr(node_id: u64, cluster_id: u64) -> String {
        format!("127.0.0.1:{}", 32000 + cluster_id * 100 + node_id)
    }

    // We can define these later in a Config.toml
    let num_clusters = 3;
    let nodes_per_cluster = 3;

    let mut all_nodes = Vec::new();
    let mut node_map = HashMap::new(); // Map to store address to RaftNode mapping

    // Start clusters
    for cluster_id in 1..=num_clusters {
        let mut cluster_nodes = Vec::new();
        let temp_dirs: Vec<_> = (1..=nodes_per_cluster)
            .map(|_| tempfile::TempDir::new().unwrap())
            .collect();

        let handle = Handle::current();
        for node_id in 1..=nodes_per_cluster {
            let handle_clone = handle.clone();
            let temp_dir = temp_dirs[node_id as usize - 1].path().to_path_buf();
            let addr = get_addr(node_id, cluster_id);
            let rpc_addr = get_rpc_addr(node_id, cluster_id);
            let addr_clone = addr.clone();
            thread::spawn(move || {
                let x = handle_clone.block_on(start_example_raft_node(
                    node_id,
                    &temp_dir,
                    addr_clone,
                    rpc_addr,
                ));
                println!("x: {:?}", x);
            });
            cluster_nodes.push((addr, 1.0));
        }
        all_nodes.push(cluster_nodes);
    }

    // Wait for servers to start up.
    tokio::time::sleep(Duration::from_millis(1_000)).await;

    // Initialize each cluster
    for (cluster_id, nodes) in all_nodes.iter().enumerate() {
        let leader = RaftNode::new(1, nodes[0].0.clone());
        println!("=== init cluster {} with leader at {}", cluster_id + 1, nodes[0].0);
        leader.init().await?;
        for (node_id, node) in nodes.iter().enumerate().skip(1) {
            println!("=== add node {} to cluster {}", node_id + 1, cluster_id + 1);
            leader.add_learner((node_id as u64 + 1, node.0.clone(), get_rpc_addr(node_id as u64 + 1, cluster_id as u64 + 1))).await?;
        }
        println!("=== change-membership for cluster {}", cluster_id + 1);
        leader.change_membership(&nodes.iter().enumerate().map(|(id, _)| id as u64 + 1).collect()).await?;
    
        node_map.insert(nodes[0].0.clone(), leader);
    }

    // Create a CARP ring with the initial leaders
    // TODO: this needs to be changed bc leaders change
    let initial_load = 1.0 / num_clusters as f32;
    let carp_ring = Carp::new(
        all_nodes.iter().map(|cluster: &Vec<(String, f64)>| {
            let (leader_addr, _) = &cluster[0];
            (leader_addr.clone(), initial_load)
        }).collect(),
        0,
    );

    // Display the CARP ring nodes
    for node in &carp_ring.nodes {
        println!("Leader Node Address: {}, Load Factor: {}", node.addr, node.load_factor);
    }

    let _ = write_to_responsible_node(&carp_ring, &node_map, "key", "value").await;
    let _ = write_to_responsible_node(&carp_ring, &node_map, "hi", "test").await;
    let _ = write_to_responsible_node(&carp_ring, &node_map, "hello", "testing").await;

    let _read_from_responsible_node = read_from_responsible_node(&carp_ring, &node_map, "hello").await;

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
