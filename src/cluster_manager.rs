use std::time::Duration;

use crate::raft_node::RaftNode;
use crate::start_example_raft_node;
use crate::carp::Carp;
use std::collections::HashMap;

use std::fs;
use std::error::Error;

use tokio::sync::watch::Sender;
use tokio::task::JoinHandle;
use tokio::sync::watch;

use toml;
use serde::Deserialize;
use serde_json;

pub struct ClusterManager {
    shutdown_channels: Vec<Sender<()>>,
    pub handles: Vec<JoinHandle<()>>,
}

#[derive(Deserialize)]
struct Config {
    num_clusters: usize,
    nodes_per_cluster: usize,
}

impl ClusterManager {
    pub async fn new(cluster_config_path: &str) -> Result<Self, Box<dyn Error>> {

        let config_contents = fs::read_to_string(cluster_config_path)?;
        let config: Config = toml::from_str(&config_contents)?;

        let num_clusters = config.num_clusters;
        let nodes_per_cluster = config.nodes_per_cluster;

        println!("Number of clusters: {}", num_clusters);
        println!("Nodes per cluster: {}", nodes_per_cluster);

        let mut handles = Vec::new();
        let mut shutdown_channels = Vec::new();

        fn get_addr(node_id: u64, cluster_id: u64) -> String {
            format!("127.0.0.1:{}", 31000 + cluster_id * 10 + node_id)
        }
        
        fn get_rpc_addr(node_id: u64, cluster_id: u64) -> String {
            format!("127.0.0.1:{}", 32000 + cluster_id * 10 + node_id)
        }

        let mut all_nodes = Vec::new();
        let mut node_map = HashMap::new();

        // Start clusters
        for cluster_id in 1..=num_clusters as u64 {
            let mut cluster_nodes = Vec::new();
            let temp_dirs: Vec<_> = (1..=nodes_per_cluster)
                .map(|_| tempfile::TempDir::new().unwrap())
                .collect();


            for node_id in 1..=nodes_per_cluster as u64 {
                let temp_dir = temp_dirs[node_id as usize - 1].path().to_path_buf();

                let addr = get_addr(node_id, cluster_id);            
                let rpc_addr = get_rpc_addr(node_id, cluster_id);
                let addr_clone = addr.clone();

                let (shutdown_tx, shutdown_rx) = watch::channel(());
                shutdown_channels.push(shutdown_tx);

                let handle = tokio::spawn(async move {
                    let _ = start_example_raft_node(node_id, &temp_dir, addr_clone, rpc_addr, shutdown_rx).await;
                });
                handles.push(handle);
                cluster_nodes.push(addr);
            }
            all_nodes.push(cluster_nodes);
        }

        // Wait for servers to start up.
        tokio::time::sleep(Duration::from_millis(1_000)).await;

        // Create a CARP ring with the initial leaders
        let initial_load = 1.0 / num_clusters as f32;
        let carp_ring = Carp::new(
            all_nodes.iter().map(|cluster| {
                let leader_addr = &cluster[0];
                (leader_addr.clone(), initial_load)
            }).collect(),
            0,
            // None
        );

        // Initialize each cluster
        for (cluster_id, nodes) in all_nodes.iter().enumerate() {
            let leader = RaftNode::new(1, nodes[0].clone());
            println!("=== init cluster {} with leader at {}", cluster_id + 1, nodes[0]);
            leader.init().await?;
            for (node_id, node) in nodes.iter().enumerate().skip(1) {
                println!("=== add node {} to cluster {}", node_id + 1, cluster_id + 1);
                leader.add_learner((node_id as u64 + 1, node.clone(), get_rpc_addr(node_id as u64 + 1, cluster_id as u64 + 1))).await?;
            }
            println!("=== change-membership for cluster {}", cluster_id + 1);
            leader.change_membership(&nodes.iter().enumerate().map(|(id, _)| id as u64 + 1).collect()).await?;
            let _ = leader.update_hash_ring(carp_ring.clone()).await;
            node_map.insert(nodes[0].clone(), leader);
        }

        let serialized_all_nodes = serde_json::to_string(&all_nodes)?;
        fs::write("all_nodes.json", serialized_all_nodes)?;
    
        Ok(ClusterManager {
            shutdown_channels: shutdown_channels,
            handles: handles,
        })
    }

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn Error>> {

        // Signal shutdown to all nodes
        for shutdown_tx in &self.shutdown_channels {
            shutdown_tx.send(()).unwrap();
        }
        
        // Wait for all nodes to shutdown
        let handles = std::mem::take(&mut self.handles);
        let _ = futures::future::join_all(handles).await;

        Ok(())
    }
}