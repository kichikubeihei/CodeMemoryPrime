use crate::handoff::SessionHandoff;
use crate::solution_vault::SolutionRecord;
use crate::failure_vault::FailureRecord;
use crate::knowledge_graph::{KnowledgeNode, KnowledgeEdge};
use crate::research_vault::ResearchRecord;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryDeltaPackage {
    pub device_source: String,
    pub exported_at: String,
    pub session_handoffs: Vec<SessionHandoff>,
    pub solution_vault: Vec<SolutionRecord>,
    pub failure_vault: Vec<FailureRecord>,
    pub knowledge_nodes: Vec<KnowledgeNode>,
    pub knowledge_edges: Vec<KnowledgeEdge>,
    #[serde(default)]
    pub research_vault: Vec<ResearchRecord>,
    pub hmac_signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MergeReport {
    pub handoffs_merged: usize,
    pub solutions_merged: usize,
    pub dead_ends_merged: usize,
    pub nodes_merged: usize,
    pub edges_merged: usize,
    pub research_merged: usize,
    pub status: String,
}

pub fn export_memory_delta(
    conn: &Connection,
    device_name: &str,
) -> Result<MemoryDeltaPackage> {
    // 1. Export Session Handoffs
    let mut handoffs = Vec::new();
    let mut h_stmt = conn.prepare(
        "SELECT project_name, task_goal, completed_steps, open_questions, active_files, timestamp 
         FROM session_handoffs"
    )?;
    let h_rows = h_stmt.query_map([], |row| {
        let completed_str: String = row.get(2)?;
        let questions_str: String = row.get(3)?;
        let files_str: String = row.get(4)?;
        Ok(SessionHandoff {
            project_name: row.get(0)?,
            task_goal: row.get(1)?,
            completed_steps: serde_json::from_str(&completed_str).unwrap_or_default(),
            open_questions: serde_json::from_str(&questions_str).unwrap_or_default(),
            active_files: serde_json::from_str(&files_str).unwrap_or_default(),
            timestamp: row.get(5)?,
        })
    })?;
    for r in h_rows {
        handoffs.push(r?);
    }

    // 2. Export Solution Vault
    let mut solutions = Vec::new();
    if let Ok(mut s_stmt) = conn.prepare(
        "SELECT id, problem_signature, domain_language, winning_model, solution_code, 
                context_and_constraints, objective_metrics, staleness_score, staleness_decay_days, 
                created_at, last_verified_at, access_count, hmac_signature, is_stale 
         FROM solution_vault"
    ) {
        let s_rows = s_stmt.query_map([], |row| {
            let metrics_str: String = row.get(6)?;
            Ok(SolutionRecord {
                id: row.get(0)?,
                problem_signature: row.get(1)?,
                domain_language: row.get(2)?,
                winning_model: row.get(3)?,
                solution_code: row.get(4)?,
                context_and_constraints: row.get(5)?,
                objective_metrics: serde_json::from_str(&metrics_str).unwrap_or(crate::solution_vault::ObjectiveMetrics {
                    compiler_exit_code: 0,
                    tests_passed: "100%".to_string(),
                    ast_complexity: 1,
                    execution_time_ms: 10.0,
                    security_clean: true,
                    zero_cls_a11y_score: 100,
                }),
                staleness_score: row.get(7)?,
                staleness_decay_days: row.get(8)?,
                created_at: row.get(9)?,
                last_verified_at: row.get(10)?,
                access_count: row.get(11)?,
                hmac_signature: row.get(12)?,
                is_stale: row.get::<_, i32>(13)? != 0,
            })
        })?;
        for r in s_rows {
            solutions.push(r?);
        }
    }

    // 3. Export Failure Vault
    let mut dead_ends = Vec::new();
    if let Ok(mut f_stmt) = conn.prepare(
        "SELECT id, failure_signature, domain_language, attempted_approach, error_message_or_trace, 
                root_cause_analysis, tested_against_environment, fatal_severity, suggested_alternative, 
                staleness_decay_days, created_at, last_attempted_at, attempt_count, hmac_signature, 
                staleness_score, is_stale_for_retest, advisory 
         FROM failure_vault"
    ) {
        let f_rows = f_stmt.query_map([], |row| {
            Ok(FailureRecord {
                id: row.get(0)?,
                failure_signature: row.get(1)?,
                domain_language: row.get(2)?,
                attempted_approach: row.get(3)?,
                error_message_or_trace: row.get(4)?,
                root_cause_analysis: row.get(5)?,
                tested_against_environment: row.get(6)?,
                fatal_severity: row.get(7)?,
                suggested_alternative: row.get(8)?,
                staleness_decay_days: row.get(9)?,
                created_at: row.get(10)?,
                last_attempted_at: row.get(11)?,
                attempt_count: row.get(12)?,
                hmac_signature: row.get(13)?,
                staleness_score: row.get(14)?,
                is_stale_for_retest: row.get::<_, i32>(15)? != 0,
                advisory: row.get(16)?,
            })
        })?;
        for r in f_rows {
            dead_ends.push(r?);
        }
    }

    // 4. Export Knowledge Graph Nodes & Edges
    let mut nodes = Vec::new();
    if let Ok(mut n_stmt) = conn.prepare(
        "SELECT id, profile, entity_type, name, content, metadata_json, created_at, updated_at 
         FROM knowledge_nodes"
    ) {
        let n_rows = n_stmt.query_map([], |row| {
            Ok(KnowledgeNode {
                id: row.get(0)?,
                profile: row.get(1)?,
                entity_type: row.get(2)?,
                name: row.get(3)?,
                content: row.get(4)?,
                metadata_json: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;
        for r in n_rows {
            nodes.push(r?);
        }
    }

    let mut edges = Vec::new();
    if let Ok(mut e_stmt) = conn.prepare(
        "SELECT id, profile, source_id, target_id, relation_type, intensity, metadata_json, created_at 
         FROM knowledge_edges"
    ) {
        let e_rows = e_stmt.query_map([], |row| {
            Ok(KnowledgeEdge {
                id: row.get(0)?,
                profile: row.get(1)?,
                source_id: row.get(2)?,
                target_id: row.get(3)?,
                relation_type: row.get(4)?,
                intensity: row.get(5)?,
                metadata_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        for r in e_rows {
            edges.push(r?);
        }
    }

    let mut research = Vec::new();
    if let Ok(mut r_stmt) = conn.prepare(
        "SELECT id, media_url, title, media_type, target_project, key_takeaways, proposed_upgrades, hmac_signature, created_at 
         FROM research_vault"
    ) {
        let r_rows = r_stmt.query_map([], |row| {
            Ok(ResearchRecord {
                id: row.get(0)?,
                media_url: row.get(1)?,
                title: row.get(2)?,
                media_type: row.get(3)?,
                target_project: row.get(4)?,
                key_takeaways: row.get(5)?,
                proposed_upgrades: row.get(6)?,
                hmac_signature: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        for r in r_rows {
            research.push(r?);
        }
    }

    let exported_at = chrono::Utc::now().to_rfc3339();
    let mut hasher = Sha256::new();
    hasher.update(device_name.as_bytes());
    hasher.update(exported_at.as_bytes());
    hasher.update(handoffs.len().to_string().as_bytes());
    hasher.update(solutions.len().to_string().as_bytes());
    hasher.update(research.len().to_string().as_bytes());
    let sig = format!("{:x}", hasher.finalize());

    Ok(MemoryDeltaPackage {
        device_source: device_name.to_string(),
        exported_at,
        session_handoffs: handoffs,
        solution_vault: solutions,
        failure_vault: dead_ends,
        knowledge_nodes: nodes,
        knowledge_edges: edges,
        research_vault: research,
        hmac_signature: sig,
    })
}

pub fn import_memory_delta(
    conn: &Connection,
    package: &MemoryDeltaPackage,
) -> Result<MergeReport> {
    // 1. Merge Handoffs
    let mut h_count = 0;
    for h in &package.session_handoffs {
        let completed_json = serde_json::to_string(&h.completed_steps).unwrap_or_default();
        let questions_json = serde_json::to_string(&h.open_questions).unwrap_or_default();
        let files_json = serde_json::to_string(&h.active_files).unwrap_or_default();

        conn.execute(
            "INSERT INTO session_handoffs (project_name, task_goal, completed_steps, open_questions, active_files, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(project_name) DO UPDATE SET
                 task_goal=excluded.task_goal,
                 completed_steps=excluded.completed_steps,
                 open_questions=excluded.open_questions,
                 active_files=excluded.active_files,
                 timestamp=excluded.timestamp",
            params![h.project_name, h.task_goal, completed_json, questions_json, files_json, h.timestamp],
        )?;
        h_count += 1;
    }

    // 2. Merge Solution Vault
    let mut s_count = 0;
    for s in &package.solution_vault {
        let metrics_json = serde_json::to_string(&s.objective_metrics).unwrap_or_default();
        conn.execute(
            "INSERT INTO solution_vault (id, problem_signature, domain_language, winning_model, solution_code, context_and_constraints, objective_metrics, staleness_score, staleness_decay_days, created_at, last_verified_at, access_count, hmac_signature, is_stale)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                 last_verified_at=excluded.last_verified_at,
                 access_count=solution_vault.access_count + 1",
            params![
                s.id, s.problem_signature, s.domain_language, s.winning_model, s.solution_code,
                s.context_and_constraints, metrics_json, s.staleness_score, s.staleness_decay_days,
                s.created_at, s.last_verified_at, s.access_count, s.hmac_signature, if s.is_stale { 1 } else { 0 }
            ],
        )?;
        s_count += 1;
    }

    // 3. Merge Failure Vault
    let mut f_count = 0;
    for f in &package.failure_vault {
        conn.execute(
            "INSERT INTO failure_vault (
                id, failure_signature, domain_language, attempted_approach,
                error_message_or_trace, root_cause_analysis, tested_against_environment,
                fatal_severity, suggested_alternative, staleness_decay_days,
                created_at, last_attempted_at, attempt_count, hmac_signature
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(id) DO UPDATE SET
                attempt_count = failure_vault.attempt_count + 1,
                last_attempted_at = excluded.last_attempted_at",
            params![
                f.id,
                f.failure_signature,
                f.domain_language,
                f.attempted_approach,
                f.error_message_or_trace,
                f.root_cause_analysis,
                f.tested_against_environment,
                f.fatal_severity,
                f.suggested_alternative,
                f.staleness_decay_days,
                f.created_at,
                f.last_attempted_at,
                f.attempt_count,
                f.hmac_signature
            ],
        )?;
        f_count += 1;
    }

    // 4. Merge Knowledge Graph Nodes
    let mut n_count = 0;
    for n in &package.knowledge_nodes {
        crate::knowledge_graph::insert_knowledge_node(conn, n)?;
        n_count += 1;
    }

    // 5. Merge Knowledge Graph Edges
    let mut e_count = 0;
    for e in &package.knowledge_edges {
        crate::knowledge_graph::insert_knowledge_edge(conn, e)?;
        e_count += 1;
    }

    // 6. Merge Research Vault
    let mut r_count = 0;
    let _ = crate::research_vault::init_research_vault_tables(conn);
    for r in &package.research_vault {
        conn.execute(
            "INSERT INTO research_vault (id, media_url, title, media_type, target_project, key_takeaways, proposed_upgrades, hmac_signature, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                 title=excluded.title,
                 key_takeaways=excluded.key_takeaways,
                 proposed_upgrades=excluded.proposed_upgrades,
                 hmac_signature=excluded.hmac_signature",
            params![
                r.id,
                r.media_url,
                r.title,
                r.media_type,
                r.target_project,
                r.key_takeaways,
                r.proposed_upgrades,
                r.hmac_signature,
                r.created_at
            ],
        )?;
        r_count += 1;
    }

    Ok(MergeReport {
        handoffs_merged: h_count,
        solutions_merged: s_count,
        dead_ends_merged: f_count,
        nodes_merged: n_count,
        edges_merged: e_count,
        research_merged: r_count,
        status: "SUCCESS".to_string(),
    })
}
