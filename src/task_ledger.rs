use crate::db::init_database;
use crate::get_db_path;
use crate::memory_integrity::compute_hmac_signature;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGateInput {
    pub gate_name: String,
    pub command: String,
    pub expected_exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGateStatus {
    pub id: String,
    pub project_name: String,
    pub task_name: String,
    pub gate_name: String,
    pub command: String,
    pub expected_exit_code: i32,
    pub passed: bool,
    pub execution_output: String,
    pub hmac_signature: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskLedgerSummary {
    pub project_name: String,
    pub total_gates: usize,
    pub passed_gates: usize,
    pub pending_gates: usize,
    pub all_gates_passed: bool,
    pub gates: Vec<TaskGateStatus>,
}

/// Locks machine-checkable task gates into SQLite task_ledger
pub fn lock_task_gates(
    project_name: &str,
    task_name: &str,
    gates: Vec<TaskGateInput>,
) -> Result<Vec<TaskGateStatus>, String> {
    let db_path = get_db_path();
    init_database(&db_path).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut locked_gates = Vec::new();

    for input in gates {
        let id = Uuid::new_v4().to_string();
        let expected_code = input.expected_exit_code.unwrap_or(0);
        let payload = format!("{}:{}:{}:{}:{}", project_name, task_name, input.gate_name, input.command, expected_code);
        let hmac = compute_hmac_signature(&payload);

        let status = TaskGateStatus {
            id: id.clone(),
            project_name: project_name.to_string(),
            task_name: task_name.to_string(),
            gate_name: input.gate_name.clone(),
            command: input.command.clone(),
            expected_exit_code: expected_code,
            passed: false,
            execution_output: "Pending Verification".to_string(),
            hmac_signature: hmac.clone(),
            timestamp: timestamp.clone(),
        };

        conn.execute(
            "INSERT OR REPLACE INTO task_ledger (id, project_name, task_name, gate_name, command, expected_exit_code, passed, execution_output, hmac_signature, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?9)",
            params![
                id,
                project_name,
                task_name,
                input.gate_name,
                input.command,
                expected_code,
                "Pending Verification",
                hmac,
                timestamp
            ],
        ).map_err(|e| e.to_string())?;

        locked_gates.push(status);
    }

    Ok(locked_gates)
}

/// Executes gate command natively in Rust, verifies exit code, and signs gate completion with HMAC-SHA256 signature
pub fn verify_gate_completion(project_name: &str, gate_id: &str) -> Result<TaskGateStatus, String> {
    let db_path = get_db_path();
    init_database(&db_path).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT task_name, gate_name, command, expected_exit_code FROM task_ledger WHERE id = ?1 AND project_name = ?2")
        .map_err(|e| e.to_string())?;

    let row = stmt
        .query_row(params![gate_id, project_name], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i32>(3)?,
            ))
        })
        .map_err(|e| format!("Gate ID '{}' not found: {}", gate_id, e))?;

    let (task_name, gate_name, command, expected_code) = row;

    let shell = if cfg!(target_os = "macos") { "zsh" } else { "sh" };
    let output = match Command::new(shell).arg("-c").arg(&command).output() {
        Ok(out) => {
            let combined = format!("{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
            let code = out.status.code().unwrap_or(-1);
            (code, combined)
        }
        Err(e) => (-1, format!("Execution Error: {}", e)),
    };

    let actual_code = output.0;
    let raw_logs = output.1;
    let bounded_logs = crate::terminal_bounder::bound_terminal_output(&raw_logs, 3000);
    let passed = actual_code == expected_code;

    let timestamp = chrono::Utc::now().to_rfc3339();
    let payload = format!("{}:{}:{}:{}:{}:PASSED={}", project_name, task_name, gate_name, command, actual_code, passed);
    let hmac = compute_hmac_signature(&payload);

    let pass_flag = if passed { 1 } else { 0 };

    conn.execute(
        "UPDATE task_ledger SET passed = ?1, execution_output = ?2, hmac_signature = ?3, timestamp = ?4 WHERE id = ?5",
        params![pass_flag, bounded_logs, hmac, timestamp, gate_id],
    ).map_err(|e| e.to_string())?;

    Ok(TaskGateStatus {
        id: gate_id.to_string(),
        project_name: project_name.to_string(),
        task_name,
        gate_name,
        command,
        expected_exit_code: expected_code,
        passed,
        execution_output: bounded_logs,
        hmac_signature: hmac,
        timestamp,
    })
}

/// Retrieves the status of all task gates for a project
pub fn get_task_ledger_status(project_name: &str) -> Result<TaskLedgerSummary, String> {
    let db_path = get_db_path();
    init_database(&db_path).map_err(|e| e.to_string())?;
    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, task_name, gate_name, command, expected_exit_code, passed, execution_output, hmac_signature, timestamp FROM task_ledger WHERE project_name = ?1 OR ?1 = 'all'")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map(params![project_name], |r| {
            let pass_int: i32 = r.get(5)?;
            Ok(TaskGateStatus {
                id: r.get(0)?,
                project_name: project_name.to_string(),
                task_name: r.get(1)?,
                gate_name: r.get(2)?,
                command: r.get(3)?,
                expected_exit_code: r.get(4)?,
                passed: pass_int == 1,
                execution_output: r.get(6)?,
                hmac_signature: r.get(7)?,
                timestamp: r.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut gates = Vec::new();
    let mut passed_gates = 0;
    for r in rows {
        if let Ok(g) = r {
            if g.passed {
                passed_gates += 1;
            }
            gates.push(g);
        }
    }

    let total_gates = gates.len();
    let pending_gates = total_gates.saturating_sub(passed_gates);
    let all_passed = total_gates > 0 && pending_gates == 0;

    Ok(TaskLedgerSummary {
        project_name: project_name.to_string(),
        total_gates,
        passed_gates,
        pending_gates,
        all_gates_passed: all_passed,
        gates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_ledger_flow() {
        let proj = format!("test_proj_{}", Uuid::new_v4());
        let gates = vec![TaskGateInput {
            gate_name: "test_cargo_check".to_string(),
            command: "echo 'Gate Passed'".to_string(),
            expected_exit_code: Some(0),
        }];

        let locked = lock_task_gates(&proj, "Test Task", gates).unwrap();
        assert_eq!(locked.len(), 1);

        let gate_id = &locked[0].id;
        let verified = verify_gate_completion(&proj, gate_id).unwrap();
        assert!(verified.passed);

        let summary = get_task_ledger_status(&proj).unwrap();
        assert!(summary.all_gates_passed);
    }
}
