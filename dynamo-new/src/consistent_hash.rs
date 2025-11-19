use md5::{Digest, Md5};

fn md5_bytes(s: &str) -> [u8; 16] {
    let mut hasher = Md5::new();
    hasher.update(s.as_bytes());
    let res = hasher.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&res);
    out
}

#[derive(Clone)]
pub struct ConsistentHash {
    ring: Vec<([u8; 16], String)>,
    hashes: Vec<[u8; 16]>,
}

impl ConsistentHash {
    pub fn new(nodes: &[String], repeat: usize) -> Self {
        let mut entries: Vec<([u8; 16], String)> = Vec::new();
        for n in nodes {
            for i in 0..repeat {
                let s = format!("{}:{}", n, i);
                entries.push((md5_bytes(&s), n.clone()));
            }
        }
        entries.sort_by(|a,b| a.0.cmp(&b.0));
        let hashes = entries.iter().map(|(h,_)| *h).collect();
        Self { ring: entries, hashes }
    }

    pub fn find_nodes(&self, key: &str, count: usize, avoid: &[String]) -> (Vec<String>, Vec<String>) {
        let keyh = md5_bytes(key);
        // binary search
        let idx = match self.hashes.binary_search_by(|probe| probe.cmp(&keyh)) {
            Ok(i) => i,
            Err(i) => i,
        };
        let mut i = idx;
        let mut results: Vec<String> = Vec::new();
        let mut avoided: Vec<String> = Vec::new();
        let avoid_set: std::collections::HashSet<&String> = avoid.iter().collect();
        while results.len() < count && !self.ring.is_empty() {
            if i == self.ring.len() { i = 0; }
            let node = &self.ring[i].1;
            if avoid_set.contains(node) {
                if !avoided.contains(node) { avoided.push(node.clone()); }
            } else if !results.contains(node) {
                results.push(node.clone());
            }
            i += 1;
            if i == idx { break; }
        }
        (results, avoided)
    }

    /// Add a new node to the consistent hash ring
    pub fn add_node(&mut self, node: &str, repeat: usize) {
        let mut new_entries: Vec<([u8; 16], String)> = Vec::new();
        for i in 0..repeat {
            let s = format!("{}:{}", node, i);
            new_entries.push((md5_bytes(&s), node.to_string()));
        }
        // Add new entries and re-sort the ring
        self.ring.extend(new_entries);
        self.ring.sort_by(|a, b| a.0.cmp(&b.0));
        self.hashes = self.ring.iter().map(|(h, _)| *h).collect();
    }

    /// Get all unique nodes in the ring
    pub fn get_nodes(&self) -> Vec<String> {
        let mut nodes: Vec<String> = self.ring.iter()
            .map(|(_, n)| n.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        nodes.sort();
        nodes
    }
}
