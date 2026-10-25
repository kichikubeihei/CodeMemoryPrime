//! Strategic Agent Squad Catalog (Dialectic Triad, Swarm Arena, Relay Pipeline)
//!
//! Organizes specialized agent personas into cohesive, role-bound tactical teams
//! governed by rigorous dialectic checks, worktree isolation, and deterministic test gates.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SquadArchetype {
    DialecticTriad,
    SwarmArena,
    RelayPipeline,
}

impl SquadArchetype {
    pub fn as_str(&self) -> &str {
        match self {
            SquadArchetype::DialecticTriad => "DialecticTriad",
            SquadArchetype::SwarmArena => "SwarmArena",
            SquadArchetype::RelayPipeline => "RelayPipeline",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "SwarmArena" => SquadArchetype::SwarmArena,
            "RelayPipeline" => SquadArchetype::RelayPipeline,
            _ => SquadArchetype::DialecticTriad,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SquadMember {
    pub role: String, // e.g. "builder", "critic", "tester", "researcher"
    pub persona_id: String,
    pub model_preference: String, // e.g. "qwen2.5-coder:32b", "$0.00-tailscale", "claude-sonnet-4-6"
    pub responsibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentSquad {
    pub squad_id: String,
    pub domain: String,
    pub display_name: String,
    pub archetype: SquadArchetype,
    pub conductor: String,
    pub members: Vec<SquadMember>,
    pub verification_gate_template: String,
    pub worktree_isolation: bool,
    pub generation: u32,
    pub win_count: u32,
    pub task_count: u32,
    pub squad_invariants: Vec<String>,
    pub last_deployed_at: String,
}

impl AgentSquad {
    /// Renders a concrete execution manifest for Grav / subagent conductors
    pub fn render_dispatch_manifest(&self, task_description: &str, target_path: &str) -> Value {
        json!({
            "squad_id": self.squad_id,
            "display_name": self.display_name,
            "domain": self.domain,
            "archetype": self.archetype.as_str(),
            "conductor": self.conductor,
            "generation": self.generation,
            "win_rate": format!("{}/{} ({:.1}%)", self.win_count, self.task_count, if self.task_count > 0 { (self.win_count as f64 / self.task_count as f64) * 100.0 } else { 100.0 }),
            "target_path": target_path,
            "task_description": task_description,
            "worktree_isolation": self.worktree_isolation,
            "verification_gate": self.verification_gate_template,
            "squad_invariants": self.squad_invariants,
            "execution_protocol": match self.archetype {
                SquadArchetype::DialecticTriad => "1. Proposer (Builder) implements candidate inside worktree -> 2. Adversary (Critic) drafts stress/edge-case tests -> 3. Arbiter executes verification gate & anti-slop firewall -> 4. Vault winning synthesis.",
                SquadArchetype::SwarmArena => "1. Spawn parallel ephemeral worktrees -> 2. Run distinct candidate models simultaneously at $0.00 cost -> 3. Execute compiler/benchmark gauntlet -> 4. Select top performer and discard losers.",
                SquadArchetype::RelayPipeline => "1. Phase 1 spec contract lock -> 2. Phase 2 implementation -> 3. Phase 3 DevSecOps audit & A11y verification.",
            },
            "member_roster": self.members
        })
    }
}

pub fn init_squad_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_squads (
            squad_id TEXT PRIMARY KEY,
            domain TEXT NOT NULL,
            display_name TEXT NOT NULL,
            archetype TEXT NOT NULL,
            conductor TEXT NOT NULL,
            members TEXT NOT NULL,
            verification_gate_template TEXT NOT NULL,
            worktree_isolation INTEGER NOT NULL,
            generation INTEGER DEFAULT 1,
            win_count INTEGER DEFAULT 0,
            task_count INTEGER DEFAULT 0,
            squad_invariants TEXT NOT NULL,
            last_deployed_at TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_squads_domain ON agent_squads (domain)",
        [],
    )?;
    seed_default_squads(conn)?;
    Ok(())
}

pub fn seed_default_squads(conn: &Connection) -> rusqlite::Result<()> {
    let starters = vec![
        AgentSquad {
            squad_id: "ttrpg_mechanics_triad".to_string(),
            domain: "tabletop_engine".to_string(),
            display_name: "RuleForge Headless Engine Triad".to_string(),
            archetype: SquadArchetype::DialecticTriad,
            conductor: "Grav".to_string(),
            members: vec![
                SquadMember {
                    role: "builder".to_string(),
                    persona_id: "ruleforge_headless_mathematician".to_string(),
                    model_preference: "qwen2.5-coder:32b".to_string(),
                    responsibility: "Implement zero-allocation headless rule atoms, dice distribution math, and bitwise LFSR algorithms".to_string(),
                },
                SquadMember {
                    role: "critic".to_string(),
                    persona_id: "devsecops_red_teamer".to_string(),
                    model_preference: "deepseek-r1:32b".to_string(),
                    responsibility: "Probe boundary conditions, modulo bias, recursion traps, and malformed entity descriptors".to_string(),
                },
                SquadMember {
                    role: "tester".to_string(),
                    persona_id: "uncle_bob_refactorer".to_string(),
                    model_preference: "$0.00-tailscale".to_string(),
                    responsibility: "Verify Clean Architecture boundaries, eliminate tautological test assertions, enforce zero-slop".to_string(),
                },
            ],
            verification_gate_template: "cargo test -p ruleforge-core --lib".to_string(),
            worktree_isolation: true,
            generation: 1,
            win_count: 0,
            task_count: 0,
            squad_invariants: vec![
                "All mechanical calculations must be pure and deterministic".to_string(),
                "Adversarial tests must probe extreme integer boundaries and empty payload collections".to_string(),
                "Zero heap allocation in inner query and die evaluation hotpaths".to_string(),
            ],
            last_deployed_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentSquad {
            squad_id: "bevy_gamedev_triad".to_string(),
            domain: "rust_gamedev".to_string(),
            display_name: "Bevy 0.14 ECS Game Architecture Triad".to_string(),
            archetype: SquadArchetype::DialecticTriad,
            conductor: "Grav".to_string(),
            members: vec![
                SquadMember {
                    role: "builder".to_string(),
                    persona_id: "bevy_ecs_architect".to_string(),
                    model_preference: "qwen2.5-coder:32b".to_string(),
                    responsibility: "Draft disjoint query systems, cache-friendly POD components, and clean state machine schedules".to_string(),
                },
                SquadMember {
                    role: "critic".to_string(),
                    persona_id: "devsecops_red_teamer".to_string(),
                    model_preference: "deepseek-r1:32b".to_string(),
                    responsibility: "Probe frame-update heap allocations, query aliasing panics, and entity command bottlenecks".to_string(),
                },
            ],
            verification_gate_template: "cargo check && cargo test".to_string(),
            worktree_isolation: true,
            generation: 1,
            win_count: 0,
            task_count: 0,
            squad_invariants: vec![
                "Disjoint query safety guaranteed at compile time".to_string(),
                "Zero allocations inside Update schedules".to_string(),
            ],
            last_deployed_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentSquad {
            squad_id: "svelte5_web_triad".to_string(),
            domain: "web_frontend".to_string(),
            display_name: "Svelte 5 Runes & Zero-CLS Triad".to_string(),
            archetype: SquadArchetype::DialecticTriad,
            conductor: "Grav".to_string(),
            members: vec![
                SquadMember {
                    role: "builder".to_string(),
                    persona_id: "svelte5_runes_craftsman".to_string(),
                    model_preference: "qwen2.5-coder:32b".to_string(),
                    responsibility: "Draft reactive Svelte 5 components using native Runes, semantic CSS tokens, and raw state optimization".to_string(),
                },
                SquadMember {
                    role: "critic".to_string(),
                    persona_id: "uncle_bob_refactorer".to_string(),
                    model_preference: "$0.00-tailscale".to_string(),
                    responsibility: "Audit A11y ARIA bindings, zero-CLS layout shifts, and script block length limits (< 200 LOC)".to_string(),
                },
            ],
            verification_gate_template: "npm test".to_string(),
            worktree_isolation: true,
            generation: 1,
            win_count: 0,
            task_count: 0,
            squad_invariants: vec![
                "Never use legacy $: reactive syntax; strictly use $state, $derived, $effect".to_string(),
                "Component script blocks must remain under 200 LOC".to_string(),
            ],
            last_deployed_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentSquad {
            squad_id: "secops_hardening_arena".to_string(),
            domain: "security".to_string(),
            display_name: "DevSecOps Multi-Model Hardening Arena".to_string(),
            archetype: SquadArchetype::SwarmArena,
            conductor: "Grav".to_string(),
            members: vec![
                SquadMember {
                    role: "candidate_1".to_string(),
                    persona_id: "devsecops_red_teamer".to_string(),
                    model_preference: "qwen2.5-coder:32b".to_string(),
                    responsibility: "Fast static analysis and network bind hardening".to_string(),
                },
                SquadMember {
                    role: "candidate_2".to_string(),
                    persona_id: "devsecops_red_teamer".to_string(),
                    model_preference: "deepseek-r1:32b".to_string(),
                    responsibility: "Cryptographic protocol and HMAC/Merkle tree integrity verification".to_string(),
                },
                SquadMember {
                    role: "candidate_3".to_string(),
                    persona_id: "uncle_bob_refactorer".to_string(),
                    model_preference: "llama3.3:70b".to_string(),
                    responsibility: "Architectural attack surface reduction and dependency supply chain pruning".to_string(),
                },
            ],
            verification_gate_template: "cargo test --bin cmp-pro".to_string(),
            worktree_isolation: true,
            generation: 1,
            win_count: 0,
            task_count: 0,
            squad_invariants: vec![
                "Zero cleartext secrets or sensitive prompts committed to disk".to_string(),
                "Local daemons bind strictly to 127.0.0.1 by default with bearer token auth".to_string(),
            ],
            last_deployed_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    for s in starters {
        let members_json = serde_json::to_string(&s.members).unwrap_or_else(|_| "[]".to_string());
        let inv_json = serde_json::to_string(&s.squad_invariants).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT OR IGNORE INTO agent_squads (
                squad_id, domain, display_name, archetype, conductor, members,
                verification_gate_template, worktree_isolation, generation, win_count, task_count,
                squad_invariants, last_deployed_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                s.squad_id,
                s.domain,
                s.display_name,
                s.archetype.as_str(),
                s.conductor,
                members_json,
                s.verification_gate_template,
                if s.worktree_isolation { 1 } else { 0 },
                s.generation,
                s.win_count,
                s.task_count,
                inv_json,
                s.last_deployed_at,
            ],
        )?;
    }

    Ok(())
}

pub fn get_squad(conn: &Connection, squad_id: &str) -> Result<Option<AgentSquad>> {
    let mut stmt = conn.prepare(
        "SELECT squad_id, domain, display_name, archetype, conductor, members,
                verification_gate_template, worktree_isolation, generation, win_count, task_count,
                squad_invariants, last_deployed_at
         FROM agent_squads WHERE squad_id = ?1",
    )?;

    let mut rows = stmt.query(params![squad_id])?;
    if let Some(row) = rows.next()? {
        let arch_str: String = row.get(3)?;
        let members_str: String = row.get(5)?;
        let inv_str: String = row.get(11)?;
        let is_isolated: i64 = row.get(7)?;

        Ok(Some(AgentSquad {
            squad_id: row.get(0)?,
            domain: row.get(1)?,
            display_name: row.get(2)?,
            archetype: SquadArchetype::from_str(&arch_str),
            conductor: row.get(4)?,
            members: serde_json::from_str(&members_str).unwrap_or_default(),
            verification_gate_template: row.get(6)?,
            worktree_isolation: is_isolated == 1,
            generation: row.get(8)?,
            win_count: row.get(9)?,
            task_count: row.get(10)?,
            squad_invariants: serde_json::from_str(&inv_str).unwrap_or_default(),
            last_deployed_at: row.get(12)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_squads(conn: &Connection, domain_filter: Option<&str>) -> Result<Vec<AgentSquad>> {
    let sql = if let Some(d) = domain_filter {
        format!(
            "SELECT squad_id, domain, display_name, archetype, conductor, members,
                    verification_gate_template, worktree_isolation, generation, win_count, task_count,
                    squad_invariants, last_deployed_at
             FROM agent_squads WHERE domain = {} ORDER BY generation DESC, win_count DESC",
            d
        )
    } else {
        "SELECT squad_id, domain, display_name, archetype, conductor, members,
                verification_gate_template, worktree_isolation, generation, win_count, task_count,
                squad_invariants, last_deployed_at
         FROM agent_squads ORDER BY domain ASC, generation DESC".to_string()
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let arch_str: String = row.get(3)?;
        let members_str: String = row.get(5)?;
        let inv_str: String = row.get(11)?;
        let is_isolated: i64 = row.get(7)?;

        Ok(AgentSquad {
            squad_id: row.get(0)?,
            domain: row.get(1)?,
            display_name: row.get(2)?,
            archetype: SquadArchetype::from_str(&arch_str),
            conductor: row.get(4)?,
            members: serde_json::from_str(&members_str).unwrap_or_default(),
            verification_gate_template: row.get(6)?,
            worktree_isolation: is_isolated == 1,
            generation: row.get(8)?,
            win_count: row.get(9)?,
            task_count: row.get(10)?,
            squad_invariants: serde_json::from_str(&inv_str).unwrap_or_default(),
            last_deployed_at: row.get(12)?,
        })
    })?;

    let mut results = Vec::new();
    for r in rows.flatten() {
        results.push(r);
    }
    Ok(results)
}

pub fn store_squad(conn: &Connection, squad: &AgentSquad) -> Result<()> {
    let members_json = serde_json::to_string(&squad.members)?;
    let inv_json = serde_json::to_string(&squad.squad_invariants)?;

    conn.execute(
        "INSERT OR REPLACE INTO agent_squads (
            squad_id, domain, display_name, archetype, conductor, members,
            verification_gate_template, worktree_isolation, generation, win_count, task_count,
            squad_invariants, last_deployed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            squad.squad_id,
            squad.domain,
            squad.display_name,
            squad.archetype.as_str(),
            squad.conductor,
            members_json,
            squad.verification_gate_template,
            if squad.worktree_isolation { 1 } else { 0 },
            squad.generation,
            squad.win_count,
            squad.task_count,
            inv_json,
            squad.last_deployed_at,
        ],
    )?;
    Ok(())
}

pub fn record_squad_outcome(conn: &Connection, squad_id: &str, succeeded: bool) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    if succeeded {
        conn.execute(
            "UPDATE agent_squads SET win_count = win_count + 1, task_count = task_count + 1, last_deployed_at = ?1 WHERE squad_id = ?2",
            params![now, squad_id],
        )?;
    } else {
        conn.execute(
            "UPDATE agent_squads SET task_count = task_count + 1, last_deployed_at = ?1 WHERE squad_id = ?2",
            params![now, squad_id],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_squad_catalog_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        init_squad_table(&conn).unwrap();

        // 1. Get squad
        let squad = get_squad(&conn, "ttrpg_mechanics_triad").unwrap().expect("Starter triad missing");
        assert_eq!(squad.archetype, SquadArchetype::DialecticTriad);
        assert_eq!(squad.members.len(), 3);
        assert!(squad.worktree_isolation);

        // 2. Render dispatch manifest
        let manifest = squad.render_dispatch_manifest("Refactor dice rolling parser", "crates/ruleforge-core");
        assert_eq!(manifest["squad_id"], "ttrpg_mechanics_triad");
        assert_eq!(manifest["archetype"], "DialecticTriad");
        assert!(manifest["member_roster"].as_array().unwrap().len() == 3);

        // 3. Record outcome
        record_squad_outcome(&conn, "ttrpg_mechanics_triad", true).unwrap();
        let updated = get_squad(&conn, "ttrpg_mechanics_triad").unwrap().unwrap();
        assert_eq!(updated.win_count, 1);
        assert_eq!(updated.task_count, 1);
    }
}
