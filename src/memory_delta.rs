use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorClock {
    pub node_clocks: HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            node_clocks: HashMap::new(),
        }
    }

    pub fn increment(&mut self, node_id: &str) {
        let count = self.node_clocks.entry(node_id.to_string()).or_insert(0);
        *count += 1;
    }

    pub fn merges_with(&mut self, other: &VectorClock) {
        for (node, &clock) in &other.node_clocks {
            let current = self.node_clocks.entry(node.clone()).or_insert(0);
            *current = (*current).max(clock);
        }
    }

    pub fn dominates(&self, other: &VectorClock) -> bool {
        let mut strictly_greater = false;
        for (node, &other_clock) in &other.node_clocks {
            let self_clock = self.node_clocks.get(node).copied().unwrap_or(0);
            if self_clock < other_clock {
                return false;
            }
            if self_clock > other_clock {
                strictly_greater = true;
            }
        }
        for (node, &self_clock) in &self.node_clocks {
            let other_clock = other.node_clocks.get(node).copied().unwrap_or(0);
            if self_clock > other_clock {
                strictly_greater = true;
            }
        }
        strictly_greater
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNodeHeader {
    pub id: String,
    pub parent_hash: String,
    pub content_hash: String,
    pub lamport_timestamp: u64,
    pub vector_clock: VectorClock,
    pub author_node: String,
}

impl MerkleNodeHeader {
    pub fn compute_hash(id: &str, parent_hash: &str, content: &str, timestamp: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(parent_hash.as_bytes());
        hasher.update(content.as_bytes());
        hasher.update(timestamp.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDeltaItem {
    pub header: MerkleNodeHeader,
    pub entry_type: String,
    pub user_request: String,
    pub ai_response: String,
    pub persona_id: String,
    pub project_name: String,
    pub consolidated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDeltaPackage {
    pub package_id: String,
    pub origin_device: String,
    pub created_at: String,
    pub merkle_root: String,
    pub vector_clock: VectorClock,
    pub items: Vec<MemoryDeltaItem>,
}

impl MemoryDeltaPackage {
    pub fn compute_merkle_root(items: &[MemoryDeltaItem]) -> String {
        let mut hasher = Sha256::new();
        for item in items {
            hasher.update(item.header.content_hash.as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }
}

pub enum MergeDecision {
    ApplyRemote,
    KeepLocal,
    ConflictBranch { lww_winner_is_remote: bool },
}

pub fn resolve_crdt_conflict(
    local_header: &MerkleNodeHeader,
    remote_header: &MerkleNodeHeader,
) -> MergeDecision {
    if remote_header.vector_clock.dominates(&local_header.vector_clock) {
        MergeDecision::ApplyRemote
    } else if local_header.vector_clock.dominates(&remote_header.vector_clock) {
        MergeDecision::KeepLocal
    } else {
        // Concurrent edit: 3-way Last-Write-Wins (LWW) with hash tie-breaker
        let is_remote_winner = if remote_header.lamport_timestamp != local_header.lamport_timestamp {
            remote_header.lamport_timestamp > local_header.lamport_timestamp
        } else {
            remote_header.content_hash > local_header.content_hash
        };
        MergeDecision::ConflictBranch {
            lww_winner_is_remote: is_remote_winner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_dominance() {
        let mut vc1 = VectorClock::new();
        vc1.increment("device_a");
        vc1.increment("device_a");

        let mut vc2 = VectorClock::new();
        vc2.increment("device_a");

        assert!(vc1.dominates(&vc2));
        assert!(!vc2.dominates(&vc1));
    }

    #[test]
    fn test_merkle_node_hash_deterministic() {
        let h1 = MerkleNodeHeader::compute_hash("id1", "root", "content_xyz", 1000);
        let h2 = MerkleNodeHeader::compute_hash("id1", "root", "content_xyz", 1000);
        assert_eq!(h1, h2);
    }
}
