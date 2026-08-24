use crate::get_db_path;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionHandoff {
    pub project_name: String,
    pub task_goal: String,
    pub completed_steps: Vec<String>,
    pub open_questions: Vec<String>,
    pub active_files: Vec<String>,
    pub timestamp: String,
}

pub fn save_session_handoff(handoff: &SessionHandoff) -> Result<()> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let completed_json = serde_json::to_string(&handoff.completed_steps).unwrap_or_default();
    let questions_json = serde_json::to_string(&handoff.open_questions).unwrap_or_default();
    let files_json = serde_json::to_string(&handoff.active_files).unwrap_or_default();
    let timestamp = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO session_handoffs (project_name, task_goal, completed_steps, open_questions, active_files, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(project_name) DO UPDATE SET
             task_goal=excluded.task_goal,
             completed_steps=excluded.completed_steps,
             open_questions=excluded.open_questions,
             active_files=excluded.active_files,
             timestamp=excluded.timestamp",
        params![handoff.project_name, handoff.task_goal, completed_json, questions_json, files_json, timestamp],
    )?;

    Ok(())
}

pub fn load_session_handoff(project_name: &str) -> Result<Option<SessionHandoff>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare(
        "SELECT project_name, task_goal, completed_steps, open_questions, active_files, timestamp
         FROM session_handoffs WHERE project_name = ?1",
    )?;

    let mut rows = stmt.query(params![project_name])?;

    if let Some(row) = rows.next()? {
        let proj: String = row.get(0)?;
        let goal: String = row.get(1)?;
        let completed_str: String = row.get(2)?;
        let questions_str: String = row.get(3)?;
        let files_str: String = row.get(4)?;
        let ts: String = row.get(5)?;

        let completed_steps: Vec<String> = serde_json::from_str(&completed_str).unwrap_or_default();
        let open_questions: Vec<String> = serde_json::from_str(&questions_str).unwrap_or_default();
        let active_files: Vec<String> = serde_json::from_str(&files_str).unwrap_or_default();

        Ok(Some(SessionHandoff {
            project_name: proj,
            task_goal: goal,
            completed_steps,
            open_questions,
            active_files,
            timestamp: ts,
        }))
    } else {
        Ok(None)
    }
}
