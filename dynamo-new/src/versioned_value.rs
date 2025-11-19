use crate::vector_clock::{VectorClock};
use bincode::{Decode, Encode};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode)]
pub struct VersionedValue {
    pub value: String,
    pub clock: VectorClock,
}

impl VersionedValue { pub fn new(value: String, clock: VectorClock) -> Self { Self { value, clock } } }

#[derive(Debug, Clone, Encode, Decode, Default)]
pub struct VersionedValues { pub versions: Vec<VersionedValue> }

impl VersionedValues {
    pub fn new() -> Self { Self { versions: vec![] } }
    pub fn add_version(&mut self, new_v: VersionedValue) {
        // drop versions strictly before new_v
        self.versions.retain(|v| !v.clock.happens_before(&new_v.clock));
        // dedupe: if an equal clock version with same value exists, skip
        let is_duplicate = self.versions.iter().any(|v| v.clock == new_v.clock && v.value == new_v.value);
        if is_duplicate { return; }
        // only add if not strictly before existing
        let should_add = !self.versions.iter().any(|v| new_v.clock.happens_before(&v.clock));
        if should_add { self.versions.push(new_v); }
    }
    pub fn merge(&mut self, other: &VersionedValues) { for v in &other.versions { self.add_version(v.clone()); } }
    pub fn has_conflict(&self) -> bool { self.versions.len() > 1 }
    pub fn contains(&self, vv: &VersionedValue) -> bool { self.versions.iter().any(|v| v.value == vv.value && v.clock == vv.clock) }
}
