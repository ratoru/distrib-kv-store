//! Hash ring implementation using CARP (Cache Array Routing Protocol)
//!
//! Follows implementation details outlined in this RFC:
//! https://datatracker.ietf.org/doc/html/draft-vinod-carp-v1-03#section-3.1

const CARP_PRIME: u32 = 0x62531965;

/// A node in the hash ring.
/// TODO: Currently ignores the fact that a node is a Raft cluster.
#[derive(Debug, Clone, PartialEq)]
struct Node {
    pub addr: String,
    /// A value between 0 and 1.
    pub relative_load: f32,
    /// Load factor multiplier. Calculated from relative load.
    pub load_factor: f32,
    /// The member proxy hash.
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
pub struct Carp {
    nodes: Vec<Node>,
}

impl Carp {
    /// Rebalances the ring by recalculating the load factors and relative loads.
    fn rebalance(&mut self) {
        if self.nodes.is_empty() {
            return;
        }
        // 1. Recalculate the relative loads.
        let total_load: f32 = self.nodes.iter().map(|node| node.relative_load).sum();
        for node in self.nodes.iter_mut() {
            node.relative_load /= total_load;
        }

        // 2. Recalculate the load factors.
        let num_nodes = self.nodes.len() as f32;
        self.nodes
            .sort_by(|a, b| a.relative_load.partial_cmp(&b.relative_load).unwrap());
        let mut last_load = (self.nodes[0].relative_load * num_nodes).powf(1.0 / num_nodes);
        self.nodes[0].load_factor = last_load;
        let mut running_prod = last_load;
        let mut last_relative = self.nodes[0].relative_load;
        for (i, node) in self.nodes.iter_mut().skip(1).enumerate() {
            // TODO: Figure out if it needs wrapping.
            let mut x_k =
                ((num_nodes - (i as f32)) * (node.relative_load - last_relative)) / running_prod;
            x_k += last_load.powf(num_nodes - (i as f32));
            x_k = x_k.powf(1.0 / (num_nodes - (i as f32)));
            node.load_factor = x_k;

            running_prod *= x_k;
            last_relative = node.relative_load;
            last_load = x_k;
        }
    }
    /// Creates a new hash ring from a vector of addresses and relative loads.
    pub fn new(nodes: Vec<(String, f32)>) -> Self {
        let nodes = nodes
            .into_iter()
            .map(|(addr, relative_load)| Node::new(addr, relative_load))
            .collect();
        let mut ring = Self { nodes };
        ring.rebalance();
        ring
    }

    /// Adds a new node to the hash ring.
    /// Recalculates relative loads and load factors.
    pub fn add_node(&mut self, addr: String, relative_load: f32) {
        let node = Node::new(addr, relative_load);
        self.nodes.push(node);
        self.rebalance();
    }

    /// Removes a node from the hash ring.
    /// Recalculates relative loads and load factors.
    pub fn remove_node(&mut self, addr: &str) {
        self.nodes.retain(|node| node.addr != addr);
        if !self.nodes.is_empty() {
            self.rebalance();
        }
    }

    /// Returns `true` if the ring is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use carp::Carp;
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
    /// use carp::Carp;
    ///
    /// let mut ring = Carp::new(vec![("node-1".to_string, 0.5), ("node-2".to_string(), 0.5)]);
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
    /// use carp::Carp;
    ///
    /// let mut ring = Carp::new(vec![("node-1".to_string(), 0.5), ("node-2".to_string(), 0.5)]);
    ///
    /// assert_eq!(ring.get("0"), "node1");
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
        assert_approx_eq!(ring.nodes[0].load_factor, 0.774_596);
        assert_approx_eq!(ring.nodes[1].load_factor, 1.000_000);
        assert_approx_eq!(ring.nodes[2].load_factor, 1.000_000);
    }
}
