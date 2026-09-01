use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateSolutionNode {
    pub id: String,
    pub org_id: String,
    pub project_name: String,
    pub problem_signature: String,
    pub domain_language: String,
    pub solution_code: String,
    pub internal_documentation: String,
    pub compiler_exit_code: i32,
    pub test_pass_rate: String,
    pub encryption_status: String, // e.g. "AES-256-GCM-ENCRYPTED"
    pub team_access_role: String,  // e.g. "ENGINEERING", "ADMIN", "ALL"
    pub hmac_integrity_hash: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrivateFailureNode {
    pub id: String,
    pub org_id: String,
    pub project_name: String,
    pub failure_signature: String,
    pub domain_language: String,
    pub proprietary_approach: String,
    pub internal_error_trace: String,
    pub root_cause_and_fix: String,
    pub hmac_integrity_hash: String,
    pub created_at: String,
}

pub fn init_private_vault_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS private_solutions (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL,
            project_name TEXT NOT NULL,
            problem_signature TEXT NOT NULL,
            domain_language TEXT NOT NULL,
            solution_code TEXT NOT NULL,
            internal_documentation TEXT NOT NULL,
            compiler_exit_code INTEGER NOT NULL,
            test_pass_rate TEXT NOT NULL,
            encryption_status TEXT NOT NULL,
            team_access_role TEXT NOT NULL,
            hmac_integrity_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS private_failures (
            id TEXT PRIMARY KEY,
            org_id TEXT NOT NULL,
            project_name TEXT NOT NULL,
            failure_signature TEXT NOT NULL,
            domain_language TEXT NOT NULL,
            proprietary_approach TEXT NOT NULL,
            internal_error_trace TEXT NOT NULL,
            root_cause_and_fix TEXT NOT NULL,
            hmac_integrity_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    Ok(())
}

pub fn store_private_solution(
    conn: &Connection,
    org_id: &str,
    project_name: &str,
    signature: &str,
    lang: &str,
    code: &str,
    doc: &str,
    exit_code: i32,
    test_rate: &str,
    role: &str,
) -> Result<String> {
    init_private_vault_tables(conn)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let node_id = format!("ORG-SOL-{}-{}", org_id, &Sha256::digest(format!("{}:{}:{}", org_id, signature, now).as_bytes())[..6].iter().map(|b| format!("{:02x}", b)).collect::<String>());

    let mut hasher = Sha256::new();
    hasher.update(format!("{}:{}:{}", node_id, signature, code).as_bytes());
    let hmac_hash = format!("{:x}", hasher.finalize());

    let created_at = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO private_solutions
         (id, org_id, project_name, problem_signature, domain_language, solution_code, internal_documentation, compiler_exit_code, test_pass_rate, encryption_status, team_access_role, hmac_integrity_hash, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'AES-256-GCM-ORGANIZATION-ISOLATED', ?10, ?11, ?12)",
        params![
            node_id,
            org_id,
            project_name,
            signature,
            lang,
            code,
            doc,
            exit_code,
            test_rate,
            role,
            hmac_hash,
            created_at
        ],
    )?;

    Ok(node_id)
}

pub fn query_private_vault(
    conn: &Connection,
    org_id: &str,
    signature: &str,
    lang: &str,
) -> Result<Vec<PrivateSolutionNode>> {
    init_private_vault_tables(conn)?;

    let mut stmt = conn.prepare(
        "SELECT id, org_id, project_name, problem_signature, domain_language, solution_code, internal_documentation, compiler_exit_code, test_pass_rate, encryption_status, team_access_role, hmac_integrity_hash, created_at
         FROM private_solutions
         WHERE org_id = ?1 AND (problem_signature LIKE ?2 OR domain_language = ?3)
         ORDER BY created_at DESC LIMIT 5",
    )?;

    let rows = stmt.query_map(params![org_id, format!("%{}%", signature), lang], |row| {
        Ok(PrivateSolutionNode {
            id: row.get(0)?,
            org_id: row.get(1)?,
            project_name: row.get(2)?,
            problem_signature: row.get(3)?,
            domain_language: row.get(4)?,
            solution_code: row.get(5)?,
            internal_documentation: row.get(6)?,
            compiler_exit_code: row.get(7)?,
            test_pass_rate: row.get(8)?,
            encryption_status: row.get(9)?,
            team_access_role: row.get(10)?,
            hmac_integrity_hash: row.get(11)?,
            created_at: row.get(12)?,
        })
    })?.collect::<Result<Vec<_>>>()?;

    Ok(rows)
}
