use bincode::{Decode, Encode};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, Default)]
pub struct VectorClock {
    pub clock: HashMap<String, u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockOrdering {
    Before,
    After,
    Concurrent,
    Equal,
}

impl VectorClock {
    pub fn new() -> Self {
        Self { clock: HashMap::new() }
    }

    pub fn increment(&mut self, node: &str) {
        *self.clock.entry(node.to_string()).or_insert(0) += 1;
    }

    pub fn update(&mut self, node: &str, counter: u64) {
        let e = self.clock.entry(node.to_string()).or_insert(0);
        if counter > *e { *e = counter; }
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (n, &c) in &other.clock {
            let e = self.clock.entry(n.clone()).or_insert(0);
            if c > *e { *e = c; }
        }
    }

    pub fn compare(&self, other: &VectorClock) -> ClockOrdering {
        let mut less = false;
        let mut greater = false;
        let mut nodes: HashSet<String> = self.clock.keys().cloned().collect();
        nodes.extend(other.clock.keys().cloned());
        for n in nodes {
            let a = *self.clock.get(&n).unwrap_or(&0);
            let b = *other.clock.get(&n).unwrap_or(&0);
            if a < b { less = true; }
            if a > b { greater = true; }
        }
        match (less, greater) { (true, true) => ClockOrdering::Concurrent, (true, false) => ClockOrdering::Before, (false, true) => ClockOrdering::After, (false, false) => ClockOrdering::Equal }
    }

    pub fn happens_before(&self, other: &VectorClock) -> bool { matches!(self.compare(other), ClockOrdering::Before) }

    pub fn converge<I: IntoIterator<Item = VectorClock>>(vcs: I) -> VectorClock {
        let mut r = VectorClock::new();
        for vc in vcs { r.merge(&vc); }
        r
    }
}
