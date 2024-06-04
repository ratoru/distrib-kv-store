use crate::raft_node::RaftNode;
use crate::store::Request;
use crate::carp::Carp;
use std::collections::HashMap;
use std::error::Error;
use tokio::sync::Mutex;
use rand::prelude::IteratorRandom;

pub struct KVClient {
    pub carp_ring: Carp,
    node_map: Mutex<HashMap<String, RaftNode>>,
}

impl KVClient {
    pub async fn new(nodes_config_path: &str) -> Result<Self, Box<dyn Error>> {
        let (carp_ring, node_map) = Self::setup(nodes_config_path).await;
        Ok(KVClient {
            carp_ring,
            node_map: Mutex::new(node_map),
        })
    }

    pub async fn write(& mut self, key: &str, value: &str) -> Result<(), Box<dyn Error>> {
        let node_map = self.node_map.lock().await;
        let responsible_node_addr = self.carp_ring.get(key);
        if let Some(responsible_node) = node_map.get(responsible_node_addr) {
            let res = responsible_node.write(&Request::Set {
                key: key.to_string(),
                value: value.to_string(),
            }).await;
            match res {
                Err(_e) => {
                    let mut did_write = false;
                    if let Some(followers) = self.carp_ring.get_followers(responsible_node_addr) {
                        for follower_address in followers {
                            if let Some(follower_node) = node_map.get(follower_address) {
                                let res = follower_node.write(&Request::Set {
                                    key: key.to_string(),
                                    value: value.to_string(),
                                }).await;
        
                                if res.is_ok() {
                                    self.carp_ring.clone().set_new_proxy(responsible_node_addr, follower_address);
                                    did_write = true;
                                    break;
                                }
                            }
                        }
                    }
                    let _ = if !did_write {
                        Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "RaftNode not found")))
                    } else {
                        Ok(())
                    };
                }
                Ok(_r) => {
                }
            }
            Ok(())
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "RaftNode not found")))
        }
    }

    pub async fn read(&self, key: &str) -> Result<String, Box<dyn Error>> {
        let node_map = self.node_map.lock().await;
        let responsible_node_addr = self.carp_ring.get(key);
        if let Some(responsible_node) = node_map.get(responsible_node_addr) {
            let response = responsible_node.read(&key.to_string()).await?;
            Ok(response)
        } else {
            Err(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "RaftNode not found")))
        }
    }

    async fn setup(nodes_config_path: &str) -> (Carp, HashMap<String, RaftNode>) {
        let data = std::fs::read_to_string(nodes_config_path).unwrap();
        let all_nodes: Vec<Vec<String>> = serde_json::from_str(&data).unwrap();
    
        let mut node_map = HashMap::new();
        for (_, nodes) in all_nodes.iter().enumerate() {
            let leader = RaftNode::new(1, nodes[0].clone());
            node_map.insert(nodes[0].clone(), leader);
        }
    
        let carp_ring: Carp;
        let mut rng = rand::thread_rng();
        if let Some((_, random_node)) = node_map.iter().choose(&mut rng) {
            carp_ring = random_node.get_hash_ring().await.unwrap();
        } else {
            panic!("No nodes available in the node_map to get the Carp ring.");
        }
    
        (carp_ring, node_map)
    }
}