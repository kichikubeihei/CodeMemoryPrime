use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentPersona {
    pub persona_id: String,
    pub domain: String,
    pub display_name: String,
    pub generation: u32,
    pub win_count: u32,
    pub task_count: u32,
    pub system_prompt_template: String,
    pub core_invariants: Vec<String>,
    pub tool_allowances: Vec<String>,
    pub known_blindspots: Vec<String>,
    pub last_evolved_at: String,
}

impl AgentPersona {
    /// Renders a complete, ready-to-use system prompt for subagent deployment
    pub fn render_system_prompt(&self, task_context: Option<&str>) -> String {
        let mut prompt = format!(
            "You are {}, a battle-tested specialist operating at Generation {}.\nDomain: {}\n\n",
            self.display_name, self.generation, self.domain
        );

        prompt.push_str("=== CORE MANDATORY INVARIANTS (NON-NEGOTIABLE) ===\n");
        for (idx, inv) in self.core_invariants.iter().enumerate() {
            prompt.push_str(&format!("{}. {}\n", idx + 1, inv));
        }

        if !self.known_blindspots.is_empty() {
            prompt.push_str("\n=== RECORDED BLINDSPOTS & FAILURE TRAPS TO AVOID ===\n");
            for bs in &self.known_blindspots {
                prompt.push_str(&format!("• [TRAP] {}\n", bs));
            }
        }

        prompt.push_str("\n=== SYSTEM INSTRUCTIONS & MENTAL MODEL ===\n");
        prompt.push_str(&self.system_prompt_template);
        prompt.push('\n');

        if let Some(ctx) = task_context {
            prompt.push_str("\n=== ACTIVE TASK CONTEXT ===\n");
            prompt.push_str(ctx);
            prompt.push('\n');
        }

        prompt
    }
}

pub fn init_persona_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agent_personas (
            persona_id TEXT PRIMARY KEY,
            domain TEXT NOT NULL,
            display_name TEXT NOT NULL,
            generation INTEGER DEFAULT 1,
            win_count INTEGER DEFAULT 0,
            task_count INTEGER DEFAULT 0,
            system_prompt_template TEXT NOT NULL,
            core_invariants TEXT NOT NULL,
            tool_allowances TEXT NOT NULL,
            known_blindspots TEXT NOT NULL,
            last_evolved_at TEXT NOT NULL
        )",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_personas_domain ON agent_personas (domain)",
        [],
    )?;
    seed_default_personas(conn)?;
    Ok(())
}

pub fn seed_default_personas(conn: &Connection) -> rusqlite::Result<()> {
    let starters = vec![
        AgentPersona {
            persona_id: "bevy_ecs_architect".to_string(),
            domain: "rust_gamedev".to_string(),
            display_name: "Bevy 0.14 ECS Game Architect".to_string(),
            generation: 1,
            win_count: 0,
            task_count: 0,
            system_prompt_template: "Expert Bevy 0.14 engine engineer. You structure games around data-oriented ECS principles. Systems must be lean, disjoint queries guaranteed, and heavy mutations isolated to event queues or command buffers.".to_string(),
            core_invariants: vec![
                "Zero heap allocations inside frame update systems".to_string(),
                "Disjoint query safety (never query &mut A alongside &A on overlapping entities)".to_string(),
                "State transitions driven strictly by NextState transitions and event readers".to_string(),
                "Maintain spatial cache locality using compact Plain-Old-Data components".to_string(),
            ],
            tool_allowances: vec!["read_file".to_string(), "edit_file".to_string(), "cmp_search".to_string(), "run_command".to_string()],
            known_blindspots: vec![
                "Using clone() on large mesh/texture assets inside tight query loops".to_string(),
                "Mixing heavy procedural generation logic with UI layout queries".to_string(),
            ],
            last_evolved_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentPersona {
            persona_id: "svelte5_runes_craftsman".to_string(),
            domain: "web_frontend".to_string(),
            display_name: "Svelte 5 Runes Craftsman".to_string(),
            generation: 1,
            win_count: 0,
            task_count: 0,
            system_prompt_template: "Modern Svelte 5 frontend engineer. You build hyper-performant, zero-CLS UI interfaces using native Runes. You treat components as modular visual units strictly governed by design system semantic tokens.".to_string(),
            core_invariants: vec![
                "Use native Svelte 5 Runes ($state, $derived, $effect); NEVER use legacy $: reactive syntax".to_string(),
                "Use $state.raw for large collections, AST trees, or data tables to eliminate deep proxy overhead".to_string(),
                "All visual styles must strictly bind semantic CSS variables (e.g. var(--bg-primary), var(--border-light))".to_string(),
                "Anti-monolith discipline: keep component script blocks under 200 lines of code".to_string(),
            ],
            tool_allowances: vec!["read_file".to_string(), "edit_file".to_string(), "cmp_search".to_string(), "run_command".to_string()],
            known_blindspots: vec![
                "Accidental infinite loops triggered by mutating untracked state inside $effect blocks".to_string(),
                "Hardcoding hex or rgba color codes instead of semantic theme tokens".to_string(),
            ],
            last_evolved_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentPersona {
            persona_id: "ruleforge_headless_mathematician".to_string(),
            domain: "tabletop_engine".to_string(),
            display_name: "RuleForge Headless Engine Mathematician".to_string(),
            generation: 1,
            win_count: 0,
            task_count: 0,
            system_prompt_template: "Deterministic game engine mathematician. You translate complex tabletop RPG rules into pure, zero-allocation headless rule atoms, bitwise Galois LFSR loops, and topological graph equations.".to_string(),
            core_invariants: vec![
                "All core mechanics must be implemented as pure, zero-allocation headless rule atoms".to_string(),
                "Procedural room generation must utilize bitwise reversible LFSR polynomials (0xB4, 255 cycle)".to_string(),
                "Subterranean 2D grid connectivity must anchor to site percolation threshold pc ~ 0.5927".to_string(),
                "Dice math must be mathematically uniform and eliminate modulo bias".to_string(),
            ],
            tool_allowances: vec!["read_file".to_string(), "edit_file".to_string(), "cmp_search".to_string(), "run_command".to_string()],
            known_blindspots: vec![
                "Leaking UI or VTT state into pure headless calculation atoms".to_string(),
                "Unbounded recursive maze-carving when percolation guarantees pathability".to_string(),
            ],
            last_evolved_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentPersona {
            persona_id: "devsecops_red_teamer".to_string(),
            domain: "security".to_string(),
            display_name: "DevSecOps Red Team Auditor".to_string(),
            generation: 1,
            win_count: 0,
            task_count: 0,
            system_prompt_template: "Adversarial security researcher. You audit repositories for unauthenticated socket exposure, supply-chain typo-squatting, prompt injection vulnerabilities, and local LLM telemetry leaks.".to_string(),
            core_invariants: vec![
                "Never permit unauthenticated 0.0.0.0 socket bindings; default strictly to 127.0.0.1".to_string(),
                "Ensure OLLAMA_NO_TELEMETRY=1 is active and reject wildcard CORS (OLLAMA_ORIGINS=*)".to_string(),
                "Enforce exact word-boundary regex checks for dependency typo-squatting".to_string(),
                "Verify zero plaintext prompts or credentials in log dumps or SQLite query caches".to_string(),
            ],
            tool_allowances: vec!["read_file".to_string(), "edit_file".to_string(), "cmp_search".to_string(), "cmp_audit".to_string(), "run_command".to_string()],
            known_blindspots: vec![
                "Loose substring regex matching producing false positives on legitimate crates (e.g. serde_json)".to_string(),
                "Overlooking systemd drop-in override paths when inspecting daemon configurations".to_string(),
            ],
            last_evolved_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentPersona {
            persona_id: "lore_voice_stylist".to_string(),
            domain: "creative_writing".to_string(),
            display_name: "Lore Titan Voice & Cognitive Stylist".to_string(),
            generation: 1,
            win_count: 0,
            task_count: 0,
            system_prompt_template: "Narrative architect and dialogue specialist. You craft distinctive character voices adhering to cognitive tier worldviews, subtextual friction, and anti-flattening voice ceilings.".to_string(),
            core_invariants: vec![
                "Anchor character dialogue strictly to @HoeMath 9 Cognitive Tiers (T1 through T9)".to_string(),
                "Never allow conversational flattening or generic helpful assistant voice in narrative prose".to_string(),
                "Enforce forbidden word boundaries and syntax cadence unique to each character entity".to_string(),
                "Preserve subtextual tension: characters never explain their inner motivations directly".to_string(),
            ],
            tool_allowances: vec!["read_file".to_string(), "edit_file".to_string(), "cmp_search".to_string()],
            known_blindspots: vec![
                "Letting high-tier characters sound like an academic LLM rather than a real human being".to_string(),
                "Permitting dialogue collision between two characters possessing similar archetypes".to_string(),
            ],
            last_evolved_at: chrono::Utc::now().to_rfc3339(),
        },
        AgentPersona {
            persona_id: "uncle_bob_refactorer".to_string(),
            domain: "software_craftsmanship".to_string(),
            display_name: "Clean Architecture & TDD Gatekeeper".to_string(),
            generation: 1,
            win_count: 0,
            task_count: 0,
            system_prompt_template: "Rigorous software craftsman and TDD purist. You decompose monolithic classes into single-responsibility modules, enforce strict LOC limits, and kill tautological tests via mutation analysis.".to_string(),
            core_invariants: vec![
                "Enforce strict Anti-Monolith LOC limits (UI components < 200 lines, Logic files < 250 lines)".to_string(),
                "CRAP score must remain under 30.0 for all refactored methods".to_string(),
                "Zero tautological test assertions: test suites must fail when core logic is mutated".to_string(),
                "Single Responsibility Principle: rigorously isolate transport, domain logic, and persistence".to_string(),
            ],
            tool_allowances: vec!["read_file".to_string(), "edit_file".to_string(), "cmp_search".to_string(), "cmp_gauntlet".to_string(), "run_command".to_string()],
            known_blindspots: vec![
                "Writing tests that pass regardless of internal state (mock tautologies)".to_string(),
                "Over-engineering simple utility functions into multi-layer abstract factories".to_string(),
            ],
            last_evolved_at: chrono::Utc::now().to_rfc3339(),
        },
    ];

    for p in starters {
        let inv_json = serde_json::to_string(&p.core_invariants).unwrap_or_else(|_| "[]".to_string());
        let tool_json = serde_json::to_string(&p.tool_allowances).unwrap_or_else(|_| "[]".to_string());
        let blind_json = serde_json::to_string(&p.known_blindspots).unwrap_or_else(|_| "[]".to_string());

        conn.execute(
            "INSERT OR IGNORE INTO agent_personas (
                persona_id, domain, display_name, generation, win_count, task_count,
                system_prompt_template, core_invariants, tool_allowances, known_blindspots, last_evolved_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            params![
                p.persona_id,
                p.domain,
                p.display_name,
                p.generation,
                p.win_count,
                p.task_count,
                p.system_prompt_template,
                inv_json,
                tool_json,
                blind_json,
                p.last_evolved_at,
            ],
        )?;
    }

    Ok(())
}

pub fn get_persona(conn: &Connection, persona_id: &str) -> Result<Option<AgentPersona>> {
    let mut stmt = conn.prepare(
        "SELECT persona_id, domain, display_name, generation, win_count, task_count,
                system_prompt_template, core_invariants, tool_allowances, known_blindspots, last_evolved_at
         FROM agent_personas WHERE persona_id = ?1",
    )?;

    let mut rows = stmt.query(params![persona_id])?;
    if let Some(row) = rows.next()? {
        let inv_str: String = row.get(7)?;
        let tools_str: String = row.get(8)?;
        let blind_str: String = row.get(9)?;

        Ok(Some(AgentPersona {
            persona_id: row.get(0)?,
            domain: row.get(1)?,
            display_name: row.get(2)?,
            generation: row.get(3)?,
            win_count: row.get(4)?,
            task_count: row.get(5)?,
            system_prompt_template: row.get(6)?,
            core_invariants: serde_json::from_str(&inv_str).unwrap_or_default(),
            tool_allowances: serde_json::from_str(&tools_str).unwrap_or_default(),
            known_blindspots: serde_json::from_str(&blind_str).unwrap_or_default(),
            last_evolved_at: row.get(10)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_personas(conn: &Connection, domain_filter: Option<&str>) -> Result<Vec<AgentPersona>> {
    let sql = if let Some(d) = domain_filter {
        format!(
            "SELECT persona_id, domain, display_name, generation, win_count, task_count,
                    system_prompt_template, core_invariants, tool_allowances, known_blindspots, last_evolved_at
             FROM agent_personas WHERE domain = '{}' ORDER BY generation DESC, win_count DESC",
            d
        )
    } else {
        "SELECT persona_id, domain, display_name, generation, win_count, task_count,
                system_prompt_template, core_invariants, tool_allowances, known_blindspots, last_evolved_at
         FROM agent_personas ORDER BY domain ASC, generation DESC".to_string()
    };

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let inv_str: String = row.get(7)?;
        let tools_str: String = row.get(8)?;
        let blind_str: String = row.get(9)?;

        Ok(AgentPersona {
            persona_id: row.get(0)?,
            domain: row.get(1)?,
            display_name: row.get(2)?,
            generation: row.get(3)?,
            win_count: row.get(4)?,
            task_count: row.get(5)?,
            system_prompt_template: row.get(6)?,
            core_invariants: serde_json::from_str(&inv_str).unwrap_or_default(),
            tool_allowances: serde_json::from_str(&tools_str).unwrap_or_default(),
            known_blindspots: serde_json::from_str(&blind_str).unwrap_or_default(),
            last_evolved_at: row.get(10)?,
        })
    })?;

    let mut results = Vec::new();
    for r in rows.flatten() {
        results.push(r);
    }
    Ok(results)
}

/// Recursively evolves an existing persona by adding new invariants or recorded blindspots (Google Harness^2 Loop)
pub fn evolve_persona(
    conn: &Connection,
    persona_id: &str,
    new_invariant: Option<&str>,
    new_blindspot: Option<&str>,
    prompt_tweak: Option<&str>,
) -> Result<AgentPersona> {
    let mut persona = get_persona(conn, persona_id)?
        .context(format!("Persona '{}' not found in catalog", persona_id))?;

    let mut evolved = false;
    if let Some(inv) = new_invariant {
        if !persona.core_invariants.iter().any(|i| i == inv) {
            persona.core_invariants.push(inv.to_string());
            evolved = true;
        }
    }

    if let Some(bs) = new_blindspot {
        if !persona.known_blindspots.iter().any(|b| b == bs) {
            persona.known_blindspots.push(bs.to_string());
            evolved = true;
        }
    }

    if let Some(tweak) = prompt_tweak {
        persona.system_prompt_template = tweak.to_string();
        evolved = true;
    }

    if evolved {
        persona.generation += 1;
        persona.last_evolved_at = chrono::Utc::now().to_rfc3339();

        let inv_json = serde_json::to_string(&persona.core_invariants)?;
        let blind_json = serde_json::to_string(&persona.known_blindspots)?;

        conn.execute(
            "UPDATE agent_personas SET
                generation = ?1,
                system_prompt_template = ?2,
                core_invariants = ?3,
                known_blindspots = ?4,
                last_evolved_at = ?5
             WHERE persona_id = ?6",
            params![
                persona.generation,
                persona.system_prompt_template,
                inv_json,
                blind_json,
                persona.last_evolved_at,
                persona.persona_id
            ],
        )?;
    }

    Ok(persona)
}

pub fn store_persona(conn: &Connection, persona: &AgentPersona) -> Result<()> {
    let inv_json = serde_json::to_string(&persona.core_invariants)?;
    let tool_json = serde_json::to_string(&persona.tool_allowances)?;
    let blind_json = serde_json::to_string(&persona.known_blindspots)?;

    conn.execute(
        "INSERT OR REPLACE INTO agent_personas (
            persona_id, domain, display_name, generation, win_count, task_count,
            system_prompt_template, core_invariants, tool_allowances, known_blindspots, last_evolved_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            persona.persona_id,
            persona.domain,
            persona.display_name,
            persona.generation,
            persona.win_count,
            persona.task_count,
            persona.system_prompt_template,
            inv_json,
            tool_json,
            blind_json,
            persona.last_evolved_at,
        ],
    )?;
    Ok(())
}

pub fn record_persona_outcome(conn: &Connection, persona_id: &str, succeeded: bool) -> Result<()> {
    if succeeded {
        conn.execute(
            "UPDATE agent_personas SET win_count = win_count + 1, task_count = task_count + 1 WHERE persona_id = ?1",
            params![persona_id],
        )?;
    } else {
        conn.execute(
            "UPDATE agent_personas SET task_count = task_count + 1 WHERE persona_id = ?1",
            params![persona_id],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_lifecycle_and_evolution() {
        let conn = Connection::open_in_memory().unwrap();
        init_persona_table(&conn).unwrap();

        let persona = get_persona(&conn, "bevy_ecs_architect").unwrap().expect("Bevy persona missing");
        assert_eq!(persona.domain, "rust_gamedev");
        assert_eq!(persona.generation, 1);
        assert!(persona.core_invariants.len() >= 4);

        // Test render system prompt
        let rendered = persona.render_system_prompt(Some("Refactor spatial grid"));
        assert!(rendered.contains("Bevy 0.14 ECS Game Architect"));
        assert!(rendered.contains("CORE MANDATORY INVARIANTS"));
        assert!(rendered.contains("Refactor spatial grid"));

        // Test Harness^2 Evolution
        let evolved = evolve_persona(
            &conn,
            "bevy_ecs_architect",
            Some("Always verify SystemId lifecycle when despawning dynamic systems"),
            Some("Leaking memory via dangling observer triggers"),
            None,
        ).unwrap();

        assert_eq!(evolved.generation, 2);
        assert!(evolved.core_invariants.iter().any(|i| i.contains("SystemId")));
        assert!(evolved.known_blindspots.iter().any(|b| b.contains("observer triggers")));

        // Record outcomes
        record_persona_outcome(&conn, "bevy_ecs_architect", true).unwrap();
        let updated = get_persona(&conn, "bevy_ecs_architect").unwrap().unwrap();
        assert_eq!(updated.win_count, 1);
        assert_eq!(updated.task_count, 1);
    }
}
