use crate::db::init_database;
use crate::get_db_path;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct PruneSummary {
    pub orphaned_chunks_deleted: usize,
    pub stale_handoffs_deleted: usize,
}

/// Prunes orphaned AST chunks and stale session handoffs whose source files no longer exist on disk.
pub fn clean_stale_context(repo_path: &str, project_name: &str) -> Result<PruneSummary> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    // Fetch all distinct file paths indexed for this project
    let mut stmt = conn.prepare(
        "SELECT DISTINCT file_path FROM code_chunks WHERE project_name = ?1",
    )?;

    let rows = stmt.query_map(params![project_name], |row| {
        let p: String = row.get(0)?;
        Ok(p)
    })?;

    let mut orphaned_files = Vec::new();
    for r in rows {
        if let Ok(file_path) = r {
            if !Path::new(&file_path).exists() {
                orphaned_files.push(file_path);
            }
        }
    }

    let mut orphaned_chunks_deleted = 0;
    for orphan in &orphaned_files {
        let deleted = conn.execute(
            "DELETE FROM code_chunks WHERE project_name = ?1 AND file_path = ?2",
            params![project_name, orphan],
        )?;
        orphaned_chunks_deleted += deleted;
    }

    // Clean up stale session handoffs for non-existent repos
    let stale_handoffs_deleted = if !Path::new(repo_path).exists() {
        conn.execute(
            "DELETE FROM session_handoffs WHERE project_name = ?1",
            params![project_name],
        )?
    } else {
        0
    };

    Ok(PruneSummary {
        orphaned_chunks_deleted,
        stale_handoffs_deleted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_stale_context() {
        let summary = clean_stale_context("/tmp", "test_clean_proj").unwrap();
        assert_eq!(summary.stale_handoffs_deleted, 0);
    }
}
