use crate::get_db_path;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionHandoff {
    pub project_name: String,
    pub task_goal: String,
    pub completed_steps: Vec<String>,
    pub open_questions: Vec<String>,
    pub active_files: Vec<String>,
    pub timestamp: String,
    #[serde(default)]
    pub prohibited_repetition: Vec<String>,
}

pub fn save_session_handoff(handoff: &SessionHandoff) -> Result<()> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    // Ensure prohibited_repetition column exists
    let _ = conn.execute("ALTER TABLE session_handoffs ADD COLUMN prohibited_repetition TEXT", []);

    let completed_json = serde_json::to_string(&handoff.completed_steps).unwrap_or_default();
    let questions_json = serde_json::to_string(&handoff.open_questions).unwrap_or_default();
    let files_json = serde_json::to_string(&handoff.active_files).unwrap_or_default();
    let prohibited_json = serde_json::to_string(&handoff.prohibited_repetition).unwrap_or_default();
    let timestamp = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO session_handoffs (project_name, task_goal, completed_steps, open_questions, active_files, timestamp, prohibited_repetition)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(project_name) DO UPDATE SET
             task_goal=excluded.task_goal,
             completed_steps=excluded.completed_steps,
             open_questions=excluded.open_questions,
             active_files=excluded.active_files,
             timestamp=excluded.timestamp,
             prohibited_repetition=excluded.prohibited_repetition",
        params![handoff.project_name, handoff.task_goal, completed_json, questions_json, files_json, timestamp, prohibited_json],
    )?;

    Ok(())
}

pub fn load_session_handoff(project_name: &str) -> Result<Option<SessionHandoff>> {
    let db_path = get_db_path();
    let conn = Connection::open(db_path)?;

    // Ensure prohibited_repetition column exists
    let _ = conn.execute("ALTER TABLE session_handoffs ADD COLUMN prohibited_repetition TEXT", []);

    let mut stmt = conn.prepare(
        "SELECT project_name, task_goal, completed_steps, open_questions, active_files, timestamp, prohibited_repetition
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
        let prohibited_str: Option<String> = row.get(6).ok();

        let completed_steps: Vec<String> = serde_json::from_str(&completed_str).unwrap_or_default();
        let open_questions: Vec<String> = serde_json::from_str(&questions_str).unwrap_or_default();
        let active_files: Vec<String> = serde_json::from_str(&files_str).unwrap_or_default();
        let prohibited_repetition: Vec<String> = prohibited_str
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Ok(Some(SessionHandoff {
            project_name: proj,
            task_goal: goal,
            completed_steps,
            open_questions,
            active_files,
            timestamp: ts,
            prohibited_repetition,
        }))
    } else {
        Ok(None)
    }
}
