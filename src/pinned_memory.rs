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
