use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionObservation {
    pub id: String,
    pub session_id: Option<String>,
    pub project_name: String,
    pub tool_name: String,
    pub target_file: Option<String>,
    pub summary: String,
    pub status: String,
    pub details: Option<String>,
    pub timestamp: String,
}

pub fn record_observation(
    conn: &Connection,
    session_id: Option<&str>,
    project_name: &str,
    tool_name: &str,
    target_file: Option<&str>,
    summary: &str,
    status: &str,
    details: Option<&str>,
) -> Result<SessionObservation> {
    let id = format!("OBS-{}", Uuid::new_v4().to_string().chars().take(8).collect::<String>());
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO session_observations (id, session_id, project_name, tool_name, target_file, summary, status, details, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, session_id, project_name, tool_name, target_file, summary, status, details, now],
    )?;

    Ok(SessionObservation {
        id,
        session_id: session_id.map(|s| s.to_string()),
        project_name: project_name.to_string(),
        tool_name: tool_name.to_string(),
        target_file: target_file.map(|s| s.to_string()),
        summary: summary.to_string(),
        status: status.to_string(),
        details: details.map(|s| s.to_string()),
        timestamp: now,
    })
}

pub fn query_observations_by_file(
    conn: &Connection,
    project_name: &str,
    target_file: &str,
    limit: usize,
) -> Result<Vec<SessionObservation>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, project_name, tool_name, target_file, summary, status, details, timestamp
         FROM session_observations
         WHERE project_name = ?1 AND (target_file = ?2 OR target_file LIKE ?3)
         ORDER BY timestamp DESC
         LIMIT ?4",
    )?;

    let file_pattern = format!("%{}%", target_file);
    let rows = stmt.query_map(params![project_name, target_file, file_pattern, limit as i64], |row| {
        Ok(SessionObservation {
            id: row.get(0)?,
            session_id: row.get(1)?,
            project_name: row.get(2)?,
            tool_name: row.get(3)?,
            target_file: row.get(4)?,
            summary: row.get(5)?,
            status: row.get(6)?,
            details: row.get(7)?,
            timestamp: row.get(8)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(obs) = r {
            list.push(obs);
        }
    }
    Ok(list)
}

pub fn search_observations(
    conn: &Connection,
    project_name: &str,
    query: &str,
    limit: usize,
) -> Result<Vec<SessionObservation>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, project_name, tool_name, target_file, summary, status, details, timestamp
         FROM session_observations
         WHERE project_name = ?1 AND (summary LIKE ?2 OR tool_name LIKE ?2 OR target_file LIKE ?2 OR details LIKE ?2)
         ORDER BY timestamp DESC
         LIMIT ?3",
    )?;

    let pattern = format!("%{}%", query);
    let rows = stmt.query_map(params![project_name, pattern, limit as i64], |row| {
        Ok(SessionObservation {
            id: row.get(0)?,
            session_id: row.get(1)?,
            project_name: row.get(2)?,
            tool_name: row.get(3)?,
            target_file: row.get(4)?,
            summary: row.get(5)?,
            status: row.get(6)?,
            details: row.get(7)?,
            timestamp: row.get(8)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(obs) = r {
            list.push(obs);
        }
    }
    Ok(list)
}

pub fn get_session_timeline(
    conn: &Connection,
    project_name: &str,
    limit: usize,
) -> Result<Vec<SessionObservation>> {
    let mut stmt = conn.prepare(
        "SELECT id, session_id, project_name, tool_name, target_file, summary, status, details, timestamp
         FROM session_observations
         WHERE project_name = ?1
         ORDER BY timestamp DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![project_name, limit as i64], |row| {
        Ok(SessionObservation {
            id: row.get(0)?,
            session_id: row.get(1)?,
            project_name: row.get(2)?,
            tool_name: row.get(3)?,
            target_file: row.get(4)?,
            summary: row.get(5)?,
            status: row.get(6)?,
            details: row.get(7)?,
            timestamp: row.get(8)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        if let Ok(obs) = r {
            list.push(obs);
        }
    }
    Ok(list)
}
