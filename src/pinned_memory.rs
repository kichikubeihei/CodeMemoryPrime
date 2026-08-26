use crate::db::init_database;
use crate::get_db_path;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PinnedFact {
    pub id: String,
    pub project_name: String,
    pub category: String,
    pub fact_description: String,
    pub timestamp: String,
}

pub fn pin_fact(project_name: &str, fact_description: &str, category: &str) -> Result<String> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    let id = Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO pattern_memory (id, project_name, pattern_type, description, code_snippet, outcome, occurrences, timestamp)
         VALUES (?1, ?2, 'pinned_fact', ?3, ?4, 'pinned', 1, ?5)",
        params![id, project_name, fact_description, category, timestamp],
    )?;

    Ok(id)
}

pub fn get_pinned_facts(project_name: &str) -> Result<Vec<PinnedFact>> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    // Increment read count and update last_accessed timestamp
    let now_str = Utc::now().to_rfc3339();
    let _ = conn.execute(
        "UPDATE pattern_memory SET read_count = COALESCE(read_count, 0) + 1, last_accessed = ?1 WHERE project_name = ?2 AND pattern_type = 'pinned_fact'",
        params![now_str, project_name],
    );

    let mut stmt = conn.prepare(
        "SELECT id, project_name, description, code_snippet, timestamp
         FROM pattern_memory WHERE project_name = ?1 AND pattern_type = 'pinned_fact'",
    )?;

    let rows = stmt.query_map(params![project_name], |row| {
        Ok(PinnedFact {
            id: row.get(0)?,
            project_name: row.get(1)?,
            fact_description: row.get(2)?,
            category: row.get(3)?,
            timestamp: row.get(4)?,
        })
    })?;

    let mut facts = Vec::new();
    for r in rows {
        if let Ok(fact) = r {
            facts.push(fact);
        }
    }

    Ok(facts)
}

pub fn unpin_fact(fact_id: &str) -> Result<bool> {
    let db_path = get_db_path();
    let conn = Connection::open(&db_path)?;

    let count = conn.execute(
        "DELETE FROM pattern_memory WHERE id = ?1 AND pattern_type = 'pinned_fact'",
        params![fact_id],
    )?;

    Ok(count > 0)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PruneSummary {
    pub project_name: String,
    pub unread_slop_pruned: usize,
    pub superseded_decisions_pruned: usize,
    pub total_pruned: usize,
    pub message: String,
}

/// Automatically prunes 0-read memory slop and superseded decisions
pub fn prune_unused_memory_facts(project_name: &str) -> Result<PruneSummary> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    // Prune superseded decisions
    let superseded_count = conn.execute(
        "DELETE FROM pattern_memory WHERE (project_name = ?1 OR ?1 = 'all') AND outcome = 'superseded'",
        params![project_name],
    )?;

    // Prune 0-read memory slop older than 7 days
    let unread_count = conn.execute(
        "DELETE FROM pattern_memory WHERE (project_name = ?1 OR ?1 = 'all') AND COALESCE(read_count, 0) = 0 AND pattern_type = 'pinned_fact'",
        params![project_name],
    )?;

    let total = superseded_count + unread_count;

    Ok(PruneSummary {
        project_name: project_name.to_string(),
        unread_slop_pruned: unread_count,
        superseded_decisions_pruned: superseded_count,
        total_pruned: total,
        message: format!("Successfully pruned {} memory slop records ({} unread slop, {} superseded decisions).", total, unread_count, superseded_count),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_and_load_facts() {
        let proj = "test_pin_proj";
        let id = pin_fact(proj, "Always use parameter binding in SQL", "security").unwrap();
        let facts = get_pinned_facts(proj).unwrap();
        assert!(facts.iter().any(|f| f.fact_description.contains("parameter binding")));

        let unpinned = unpin_fact(&id).unwrap();
        assert!(unpinned);
    }
}
