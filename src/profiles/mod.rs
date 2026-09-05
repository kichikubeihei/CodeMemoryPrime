use crate::knowledge_graph::KnowledgeNode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum DomainProfile {
    Code,
    Lore,
    Ttrpg,
    Agent,
}

impl DomainProfile {
    pub fn as_str(&self) -> &'static str {
        match self {
            DomainProfile::Code => "code",
            DomainProfile::Lore => "lore",
            DomainProfile::Ttrpg => "ttrpg",
            DomainProfile::Agent => "agent",
        }
    }
}

/// Lore Titan & Altalune Entity Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoreEntity {
    pub id: String,
    pub entity_type: String, // "character", "faction", "location", "magic_system", "timeline_event", "artifact"
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub canon_rules: Vec<String>,
    pub metadata: serde_json::Value,
}

impl LoreEntity {
    pub fn to_knowledge_node(&self) -> KnowledgeNode {
        let metadata_val = serde_json::json!({
            "aliases": self.aliases,
            "canon_rules": self.canon_rules,
            "extra": self.metadata
        });

        KnowledgeNode {
            id: self.id.clone(),
            profile: DomainProfile::Lore.as_str().to_string(),
            entity_type: self.entity_type.clone(),
            name: self.name.clone(),
            content: self.description.clone(),
            metadata_json: metadata_val.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            evidence_tier: "axiom".to_string(),
        }
    }
}

/// RuleForge TTRPG Entity Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TtrpgEntity {
    pub id: String,
    pub entity_type: String, // "stat_block", "spell", "condition", "dice_mechanic", "action_economy"
    pub system_edition: String, // "dnd5e", "pf2e", "custom"
    pub name: String,
    pub rules_text: String,
    pub mechanics: serde_json::Value,
}

impl TtrpgEntity {
    pub fn to_knowledge_node(&self) -> KnowledgeNode {
        let metadata_val = serde_json::json!({
            "system_edition": self.system_edition,
            "mechanics": self.mechanics
        });

        KnowledgeNode {
            id: self.id.clone(),
            profile: DomainProfile::Ttrpg.as_str().to_string(),
            entity_type: self.entity_type.clone(),
            name: self.name.clone(),
            content: self.rules_text.clone(),
            metadata_json: metadata_val.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
            evidence_tier: "axiom".to_string(),
        }
    }
}

/// AIMACS Autonomous Agent Entity Types
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentEntity {
    pub id: String,
    pub entity_type: String, // "subagent_state", "task_goal", "gate_ledger", "decision_log"
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
