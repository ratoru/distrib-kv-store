use std::backtrace::Backtrace;
use std::collections::BTreeMap;
use std::panic::PanicInfo;
use std::thread;
use std::time::Duration;

use distrib_kv_store::node::RaftNode;
use distrib_kv_store::start_example_raft_node;
use distrib_kv_store::store::Request;
use distrib_kv_store::Node;
use maplit::btreemap;
use maplit::btreeset;
use tokio::runtime::Handle;
use tracing_subscriber::EnvFilter;

use distrib_kv_store::carp::Carp;
use tokio::sync::RwLock;
use std::sync::Arc;

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
    }

    Ok(())
}