use crate::db::init_database;
use crate::get_db_path;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RefactorDecision {
    pub id: String,
    pub project_name: String,
    pub symbol_or_file: String,
    pub decision_text: String,
    pub rationale: String,
    pub timestamp: String,
}

pub fn log_decision(
    project_name: &str,
    symbol_or_file: &str,
    decision_text: &str,
    rationale: &str,
) -> Result<String> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    let id = Uuid::new_v4().to_string();
    let timestamp = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO pattern_memory (id, project_name, pattern_type, description, code_snippet, outcome, occurrences, timestamp)
         VALUES (?1, ?2, 'refactor_decision', ?3, ?4, ?5, 1, ?6)",
        params![id, project_name, decision_text, symbol_or_file, rationale, timestamp],
    )?;

    Ok(id)
}

pub fn get_decisions(project_name: &str, symbol_or_file: &str) -> Result<Vec<RefactorDecision>> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    let mut stmt = conn.prepare(
        "SELECT id, project_name, description, code_snippet, outcome, timestamp
         FROM pattern_memory
         WHERE project_name = ?1 AND pattern_type = 'refactor_decision' AND (code_snippet = ?2 OR ?2 = 'all')",
    )?;

    let rows = stmt.query_map(params![project_name, symbol_or_file], |row| {
        Ok(RefactorDecision {
            id: row.get(0)?,
            project_name: row.get(1)?,
            decision_text: row.get(2)?,
            symbol_or_file: row.get(3)?,
            rationale: row.get(4)?,
            timestamp: row.get(5)?,
        })
    })?;

    let mut decisions = Vec::new();
    for r in rows {
        if let Ok(d) = r {
            decisions.push(d);
        }
    }

    Ok(decisions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_and_get_decisions() {
        let proj = "test_adr_proj";
        let _id = log_decision(proj, "auth_service.rs", "Use Mutex over RwLock", "Prevents SQLite pool contention under heavy load").unwrap();
        let list = get_decisions(proj, "auth_service.rs").unwrap();
        assert!(list.iter().any(|d| d.decision_text.contains("Mutex")));
    }
}
