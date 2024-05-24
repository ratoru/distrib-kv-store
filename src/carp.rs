//! Hash ring implementation using CARP (Cache Array Routing Protocol)
//!
//! Follows implementation details outlined in this RFC:
//! https://datatracker.ietf.org/doc/html/draft-vinod-carp-v1-03#section-3.1
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;

const CARP_PRIME: u32 = 0x62531965;

/// A node in the hash ring.
/// TODO: Currently ignores the fact that a node is a Raft cluster.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub addr: String,
    /// A value between 0 and 1.
    pub relative_load: f32,
    /// Load factor multiplier. Calculated from relative load.
    #[serde(skip)]
    pub load_factor: f32,
    /// The member proxy hash.
    #[serde(skip)]
    pub hash: u32,
}

impl Node {
    /// Creates a new node.
    pub fn new(addr: String, relative_load: f32) -> Self {
        let hash = membership_hash(&addr);
        Self {
            addr,
            relative_load,
            load_factor: 0.0,
            hash,
        }
    }
}

/// A CARP hash ring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Carp {
    /// CARP protocol version.
    pub version: f32,
    /// Configuration ID. Used to determine if a configuration has changed.
    pub config_id: u32,
    /// List time-to-live. Used to determine how long a list of members is valid.
    pub list_ttl: u32,
    #[serde(deserialize_with = "deserialize_nodes")]
    pub nodes: Vec<Node>,
}

impl Carp {
    /// Creates a new hash ring from a vector of addresses and relative loads.
    pub fn new(nodes: Vec<(String, f32)>) -> Self {
        let nodes = nodes
            .into_iter()
            .map(|(addr, relative_load)| Node::new(addr, relative_load))
            .collect();
        let mut ring = Self {
            nodes,
            version: 1.0,
            config_id: 0,
            list_ttl: 10 * 60, // 10 minutes
        };
        rebalance(&mut ring.nodes);
        ring
    }

    /// Adds a new node to the hash ring.
    /// Recalculates relative loads and load factors.
    pub fn add_node(&mut self, addr: String, relative_load: f32) {
        let node = Node::new(addr, relative_load);
        self.nodes.push(node);
        rebalance(&mut self.nodes);
        self.config_id += 1;
    }

    /// Removes a node from the hash ring.
    /// Recalculates relative loads and load factors.
    pub fn remove_node(&mut self, addr: &str) {
        self.nodes.retain(|node| node.addr != addr);
        if !self.nodes.is_empty() {
            rebalance(&mut self.nodes);
        }
        self.config_id += 1;
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use distrib_kv_store::carp::Carp;
    ///
    /// let mut ring = Carp::new(vec![]);
    ///
    /// assert!(ring.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of nodes in the ring.
    ///
    /// # Examples
    ///
    /// ```
    /// use distrib_kv_store::carp::Carp;
    ///
    /// let mut ring = Carp::new(vec![("node-1".to_string(), 0.5), ("node-2".to_string(), 0.5)]);
    ///
    /// assert_eq!(ring.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the node responsible for the given URL.
    ///
    /// # Panics
    ///
    /// Panics if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use distrib_kv_store::carp::Carp;
    ///
    /// let mut ring = Carp::new(vec![("node-1".to_string(), 0.5), ("node-2".to_string(), 0.5)]);
    ///
    /// assert_eq!(ring.get("foo"), "node-1");
    /// ```
    pub fn get(&self, url: &str) -> &str {
        if self.is_empty() {
            panic!("Hash ring is empty");
        }
        let url_hash = url_hash(url);
        let mut best_score = f32::MIN;
        let mut best_node = &self.nodes[0];
        for node in self.nodes.iter() {
            let combined = combine_hashes(node.hash, url_hash);
            let score = (combined as f32) * node.load_factor;
            if score > best_score {
                best_score = score;
                best_node = node;
            }
        }
        best_node.addr.as_str()
    }
}

/// Calculates the membership hash for a given address.
///
/// Because irreversibility and strong cryptographic features are
/// unnecessary for this application, a very simple and fast hash
/// function based on the bitwise left rotate operator is used.
fn membership_hash(addr: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in addr.bytes() {
        let rotated = hash.rotate_left(19).wrapping_add(c as u32);
        hash = hash.wrapping_add(rotated)
    }
    hash = hash.wrapping_add(hash.wrapping_mul(CARP_PRIME));
    hash.rotate_left(21)
}

/// Calculates the hash for a given URL.
fn url_hash(url: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in url.bytes() {
        let rotated = hash.rotate_left(19).wrapping_add(c as u32);
        hash = hash.wrapping_add(rotated)
    }
    hash
}

/// Combines the membership hash with the URL hash.
fn combine_hashes(membership: u32, url: u32) -> u32 {
    let mut combined = membership ^ url;
    combined = combined.wrapping_add(combined.wrapping_mul(CARP_PRIME));
    combined.rotate_left(21)
}

/// Rebalances the ring by recalculating the load factors and relative loads.
fn rebalance(nodes: &mut Vec<Node>) {
    if nodes.is_empty() {
        return;
    }
    // 1. Recalculate the relative loads.
    let total_load: f32 = nodes.iter().map(|node| node.relative_load).sum();
    for node in nodes.iter_mut() {
        node.relative_load /= total_load;
    }
    // 2. Recalculate the load factors.
    let num_nodes = nodes.len() as f32;
    nodes.sort_by(|a, b| a.relative_load.partial_cmp(&b.relative_load).unwrap());
    let mut last_load = (nodes[0].relative_load * num_nodes).powf(1.0 / num_nodes);
    nodes[0].load_factor = last_load;
    let mut running_prod = last_load;
    let mut last_relative = nodes[0].relative_load;
    for (i, node) in nodes.iter_mut().skip(1).enumerate() {
        let mut x_k =
            ((num_nodes - ((i + 1) as f32)) * (node.relative_load - last_relative)) / running_prod;
        x_k += last_load.powf(num_nodes - ((i + 1) as f32));
        x_k = x_k.powf(1.0 / (num_nodes - ((i + 1) as f32)));
        node.load_factor = x_k;
        running_prod *= x_k;
        last_relative = node.relative_load;
        last_load = x_k;
    }
}

fn deserialize_nodes<'de, D>(deserializer: D) -> Result<Vec<Node>, D::Error>
where
    D: Deserializer<'de>,
{
    let mut nodes = Vec::<Node>::deserialize(deserializer)?;

    for node in nodes.iter_mut() {
        node.hash = membership_hash(&node.addr);
    }
    rebalance(&mut nodes);

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr) => {{
            let (a, b) = (&$a, &$b);
            assert!(
                (*a - *b).abs() < 1.0e-6,
                "{} is not approximately equal to {}",
                *a,
                *b
            );
        }};
    }

    #[test]
    fn test_size_empty() {
        let ring = Carp::new(vec![]);
        assert!(ring.is_empty());
        assert_eq!(ring.len(), 0);
    }

    #[test]
    fn test_correct_weights() {
        let ring = Carp::new(vec![
            ("0".to_string(), 0.4),
            ("1".to_string(), 0.4),
            ("2".to_string(), 0.2),
        ]);
        assert_eq!(ring.nodes[0].addr, "2");
        assert_eq!(ring.nodes[1].addr, "0");
        assert_eq!(ring.nodes[2].addr, "1");
        assert_approx_eq!(ring.nodes[0].relative_load, 0.2);
        assert_approx_eq!(ring.nodes[1].relative_load, 0.4);
        assert_approx_eq!(ring.nodes[2].relative_load, 0.4);
        assert_approx_eq!(ring.nodes[0].load_factor, 0.843_433);
        assert_approx_eq!(ring.nodes[1].load_factor, 1.088_866);
        assert_approx_eq!(ring.nodes[2].load_factor, 1.088_866);
    }

    #[test]
    fn test_add_node() {
        let mut ring = Carp::new(vec![("0".to_string(), 0.5), ("1".to_string(), 0.5)]);
        ring.add_node("2".to_string(), 0.25);
        assert_eq!(ring.len(), 3);
        // Check that rebalance works correctly.
        assert_eq!(ring.nodes[0].addr, "2");
        assert_eq!(ring.nodes[1].addr, "0");
        assert_eq!(ring.nodes[2].addr, "1");
        assert_approx_eq!(ring.nodes[0].relative_load, 0.2);
        assert_approx_eq!(ring.nodes[1].relative_load, 0.4);
        assert_approx_eq!(ring.nodes[2].relative_load, 0.4);
        assert_approx_eq!(ring.nodes[0].load_factor, 0.843_433);
        assert_approx_eq!(ring.nodes[1].load_factor, 1.088_866);
        assert_approx_eq!(ring.nodes[2].load_factor, 1.088_866);
    }

    #[test]
    fn test_remove_node() {
        let mut ring = Carp::new(vec![("0".to_string(), 0.5), ("1".to_string(), 0.5)]);
        ring.remove_node("0");
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.nodes[0].addr, "1");
        assert_approx_eq!(ring.nodes[0].relative_load, 1.0);
        assert_approx_eq!(ring.nodes[0].load_factor, 1.0);
    }

    #[test]
    fn test_serializing_carp() {
        let ring = Carp::new(vec![("0".to_string(), 0.8), ("1".to_string(), 0.2)]);
        let serialized = serde_json::to_string(&ring).unwrap();
        let deserialized: Carp = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ring.nodes[0].addr, deserialized.nodes[0].addr);
        assert_eq!(ring.nodes[1].addr, deserialized.nodes[1].addr);
        assert_approx_eq!(
            ring.nodes[0].relative_load,
            deserialized.nodes[0].relative_load
        );
        assert_approx_eq!(
            ring.nodes[1].relative_load,
            deserialized.nodes[1].relative_load
        );
        assert_approx_eq!(ring.nodes[0].load_factor, deserialized.nodes[0].load_factor);
        assert_approx_eq!(ring.nodes[1].load_factor, deserialized.nodes[1].load_factor);
    }
}
