use crate::get_db_path;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternRecord {
    pub id: String,
    pub project_name: String,
    pub pattern_type: String, // e.g. "anti_pattern", "best_practice", "fix_pattern"
    pub description: String,
    pub code_snippet: String,
    pub outcome: String,      // "success" or "failure"
    pub occurrences: i32,
    pub timestamp: String,
}

pub fn log_pattern(project_name: &str, pattern_type: &str, description: &str, snippet: &str, outcome: &str) -> Result<()> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    // Check if pattern already logged
    let mut stmt = conn.prepare(
        "SELECT id, occurrences FROM pattern_memory WHERE project_name = ?1 AND description = ?2 AND outcome = ?3",
    )?;

    let mut rows = stmt.query(params![project_name, description, outcome])?;

    if let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let occurrences: i32 = row.get(1)?;
        conn.execute(
            "UPDATE pattern_memory SET occurrences = ?1, timestamp = ?2 WHERE id = ?3",
            params![occurrences + 1, Utc::now().to_rfc3339(), id],
        )?;
    } else {
        let id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO pattern_memory (id, project_name, pattern_type, description, code_snippet, outcome, occurrences, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7)",
            params![id, project_name, pattern_type, description, snippet, outcome, Utc::now().to_rfc3339()],
        )?;
    }

    Ok(())
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<PatternRecord> {
    Ok(PatternRecord {
        id: row.get(0)?,
        project_name: row.get(1)?,
        pattern_type: row.get(2)?,
        description: row.get(3)?,
        code_snippet: row.get(4)?,
        outcome: row.get(5)?,
        occurrences: row.get(6)?,
        timestamp: row.get(7)?,
    })
}

pub fn search_patterns(project_name: &str, pattern_type: Option<&str>) -> Result<Vec<PatternRecord>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut records = Vec::new();
    if let Some(pt) = pattern_type {
        let mut stmt = conn.prepare("SELECT id, project_name, pattern_type, description, code_snippet, outcome, occurrences, timestamp FROM pattern_memory WHERE project_name = ?1 AND pattern_type = ?2")?;
        let rows = stmt.query_map(params![project_name, pt], map_row)?;
        for r in rows {
            if let Ok(rec) = r {
                records.push(rec);
            }
        }
    } else {
        let mut stmt = conn.prepare("SELECT id, project_name, pattern_type, description, code_snippet, outcome, occurrences, timestamp FROM pattern_memory WHERE project_name = ?1")?;
        let rows = stmt.query_map(params![project_name], map_row)?;
        for r in rows {
            if let Ok(rec) = r {
                records.push(rec);
            }
        }
    };

    Ok(records)
}
