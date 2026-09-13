use crate::knowledge_graph::KnowledgeNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DomainProfile {
    Code,
    Algorithm,
    Architecture,
    Verification,
    Agent,
}

impl DomainProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            DomainProfile::Code => "code",
            DomainProfile::Algorithm => "algorithm",
            DomainProfile::Architecture => "architecture",
            DomainProfile::Verification => "verification",
            DomainProfile::Agent => "agent",
        }
    }
}

/// AST Symbol & Code Structure Entity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CodeEntity {
    pub id: String,
    pub entity_type: String, // "module", "struct", "enum", "function", "trait", "interface"
    pub name: String,
    pub file_path: String,
    pub line_number: usize,
    pub signature: String,
    pub docstring: String,
    pub metadata: serde_json::Value,
}

impl CodeEntity {
    pub fn to_knowledge_node(&self) -> KnowledgeNode {
        let metadata_val = serde_json::json!({
            "file_path": self.file_path,
            "line_number": self.line_number,
            "signature": self.signature,
            "extra": self.metadata
        });

        KnowledgeNode {
            id: self.id.clone(),
            profile: DomainProfile::Code.as_str().to_string(),
            entity_type: self.entity_type.clone(),
            name: self.name.clone(),
            content: self.docstring.clone(),
            metadata_json: metadata_val.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            evidence_tier: "ast_proven".to_string(),
        }
    }
}

/// Verified Algorithm & Performance Spell Entity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlgorithmEntity {
    pub id: String,
    pub entity_type: String, // "sorting", "graph", "cache_blocking", "lock_free", "simd", "compression"
    pub name: String,
    pub complexity: String, // e.g. "O(N log N)", "O(1)"
    pub invariants: Vec<String>,
    pub implementation: String,
    pub benchmarks: serde_json::Value,
}

impl AlgorithmEntity {
    pub fn to_knowledge_node(&self) -> KnowledgeNode {
        let metadata_val = serde_json::json!({
            "complexity": self.complexity,
            "invariants": self.invariants,
            "benchmarks": self.benchmarks
        });

        KnowledgeNode {
            id: self.id.clone(),
            profile: DomainProfile::Algorithm.as_str().to_string(),
            entity_type: self.entity_type.clone(),
            name: self.name.clone(),
            content: self.implementation.clone(),
            metadata_json: metadata_val.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            evidence_tier: "kani_verified".to_string(),
        }
    }
}

/// Software System & Architecture Entity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArchitectureEntity {
    pub id: String,
    pub entity_type: String, // "subsystem", "api_contract", "pipeline", "storage_schema", "security_boundary"
    pub name: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub tech_stack: Vec<String>,
    pub metadata: serde_json::Value,
}

impl ArchitectureEntity {
    pub fn to_knowledge_node(&self) -> KnowledgeNode {
        let metadata_val = serde_json::json!({
            "dependencies": self.dependencies,
            "tech_stack": self.tech_stack,
            "extra": self.metadata
        });

        KnowledgeNode {
            id: self.id.clone(),
            profile: DomainProfile::Architecture.as_str().to_string(),
            entity_type: self.entity_type.clone(),
            name: self.name.clone(),
            content: self.description.clone(),
            metadata_json: metadata_val.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            evidence_tier: "architectural".to_string(),
        }
    }
}

/// Autonomous Coding Agent Session Entity
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentEntity {
    pub id: String,
    pub entity_type: String, // "pairing_session", "task_goal", "gate_ledger", "decision_log"
    pub name: String,
    pub status: String,
    pub current_goal: String,
    pub active_state: serde_json::Value,
}

impl AgentEntity {
    pub fn to_knowledge_node(&self) -> KnowledgeNode {
        let metadata_val = serde_json::json!({
            "status": self.status,
            "current_goal": self.current_goal,
            "active_state": self.active_state
        });

        KnowledgeNode {
            id: self.id.clone(),
            profile: DomainProfile::Agent.as_str().to_string(),
            entity_type: self.entity_type.clone(),
            name: self.name.clone(),
            content: self.current_goal.clone(),
            metadata_json: metadata_val.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            evidence_tier: "historical".to_string(),
        }
    }
}
