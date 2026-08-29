use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const FAILURE_HMAC_SECRET: &str = "CMP_FAILURE_VAULT_TAMPER_PROOF_2026";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FailureRecord {
    pub id: String,
    pub failure_signature: String,
    pub domain_language: String,
    pub attempted_approach: String,
    pub error_message_or_trace: String,
    pub root_cause_analysis: String,
    pub tested_against_environment: String,
    pub fatal_severity: String,
    pub suggested_alternative: String,
    pub staleness_decay_days: u32,
    pub created_at: String,
    pub last_attempted_at: String,
    pub attempt_count: u32,
    pub hmac_signature: String,
    pub staleness_score: f64,
    pub is_stale_for_retest: bool,
    pub advisory: String,
}

pub fn init_failure_vault_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS failure_vault (
            id TEXT PRIMARY KEY,
            failure_signature TEXT NOT NULL,
            domain_language TEXT NOT NULL,
            attempted_approach TEXT NOT NULL,
            error_message_or_trace TEXT NOT NULL,
            root_cause_analysis TEXT NOT NULL,
            tested_against_environment TEXT NOT NULL,
            fatal_severity TEXT NOT NULL,
            suggested_alternative TEXT,
            staleness_decay_days INTEGER DEFAULT 90,
            created_at TEXT NOT NULL,
            last_attempted_at TEXT NOT NULL,
            attempt_count INTEGER DEFAULT 1,
            hmac_signature TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_failure_signature ON failure_vault(failure_signature)",
        [],
    )?;

    Ok(())
}

fn compute_failure_hmac(signature: &str, approach: &str, error: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(FAILURE_HMAC_SECRET.as_bytes());
    hasher.update(signature.as_bytes());
    hasher.update(approach.as_bytes());
    hasher.update(error.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn current_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.to_string()
}

pub fn store_vault_failure(
    conn: &Connection,
    signature: &str,
    domain_language: &str,
    attempted_approach: &str,
    error_message_or_trace: &str,
    root_cause_analysis: &str,
    tested_against_environment: &str,
    fatal_severity: &str,
    suggested_alternative: &str,
    decay_days: u32,
) -> Result<String> {
    init_failure_vault_table(conn)?;

    let id = format!("FAIL-{}", &compute_failure_hmac(signature, attempted_approach, error_message_or_trace)[..12]);
    let hmac = compute_failure_hmac(signature, attempted_approach, error_message_or_trace);
    let now_ts = current_timestamp();

    conn.execute(
        "INSERT INTO failure_vault (
            id, failure_signature, domain_language, attempted_approach,
            error_message_or_trace, root_cause_analysis, tested_against_environment,
            fatal_severity, suggested_alternative, staleness_decay_days,
            created_at, last_attempted_at, attempt_count, hmac_signature
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, 1, ?13)
        ON CONFLICT(id) DO UPDATE SET
            attempt_count = failure_vault.attempt_count + 1,
            last_attempted_at = excluded.last_attempted_at,
            error_message_or_trace = excluded.error_message_or_trace,
            root_cause_analysis = excluded.root_cause_analysis,
            hmac_signature = excluded.hmac_signature",
        params![
            id,
            signature,
            domain_language,
            attempted_approach,
            error_message_or_trace,
            root_cause_analysis,
            tested_against_environment,
            fatal_severity,
            suggested_alternative,
            decay_days,
            now_ts,
            now_ts,
            hmac
        ],
    )?;

    Ok(id)
}

pub fn query_vault_dead_ends(
    conn: &Connection,
    query_term: &str,
    max_allowed_staleness: f64,
) -> Result<Vec<FailureRecord>> {
    init_failure_vault_table(conn)?;

    let mut stmt = conn.prepare(
        "SELECT id, failure_signature, domain_language, attempted_approach,
                error_message_or_trace, root_cause_analysis, tested_against_environment,
                fatal_severity, suggested_alternative, staleness_decay_days,
                created_at, last_attempted_at, attempt_count, hmac_signature
         FROM failure_vault
         WHERE failure_signature LIKE ?1 OR attempted_approach LIKE ?1
         ORDER BY last_attempted_at DESC LIMIT 5"
    )?;

    let pattern = format!("%{}%", query_term);
    let mut rows = stmt.query([pattern])?;
    let mut records = Vec::new();

    let now_sec = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let signature: String = row.get(1)?;
        let domain: String = row.get(2)?;
        let approach: String = row.get(3)?;
        let error: String = row.get(4)?;
        let root_cause: String = row.get(5)?;
        let env: String = row.get(6)?;
        let severity: String = row.get(7)?;
        let alternative: Option<String> = row.get(8)?;
        let decay_days: u32 = row.get(9)?;
        let created_at: String = row.get(10)?;
        let last_attempted: String = row.get(11)?;
        let attempt_count: u32 = row.get(12)?;
        let hmac: String = row.get(13)?;

        // 1. HMAC anti-tampering verification
        let expected_hmac = compute_failure_hmac(&signature, &approach, &error);
        if expected_hmac != hmac {
            continue; // Skip corrupted / spoofed entries
        }

        // 2. Compute staleness score
        let last_attempted_sec: u64 = last_attempted.parse().unwrap_or(now_sec);
        let age_days = (now_sec.saturating_sub(last_attempted_sec) as f64) / 86400.0;
        let decay_limit = (decay_days as f64).max(1.0);
        let staleness = (age_days / decay_limit).min(1.0);
        let is_stale = staleness > max_allowed_staleness;

        let advisory = if is_stale {
            format!(
                "⚠️ DEAD-END IS AGED (Staleness: {:.0}% > {:.0}%). Language standards, runtimes (current: {}), or compute models may have evolved. Unconventional re-testing is PERMITTED.",
                staleness * 100.0,
                max_allowed_staleness * 100.0,
                env
            )
        } else {
            format!(
                "🛑 ACTIVE DEAD-END (Staleness: {:.0}%). Fatal failure confirmed in {}. DO NOT PURSUE. Recommended alternative: {}",
                staleness * 100.0,
                domain,
                alternative.as_deref().unwrap_or("See root cause analysis")
            )
        };

        records.push(FailureRecord {
            id,
            failure_signature: signature,
            domain_language: domain,
            attempted_approach: approach,
            error_message_or_trace: error,
            root_cause_analysis: root_cause,
            tested_against_environment: env,
            fatal_severity: severity,
            suggested_alternative: alternative.unwrap_or_default(),
            staleness_decay_days: decay_days,
            created_at,
            last_attempted_at: last_attempted,
            attempt_count,
            hmac_signature: hmac,
            staleness_score: (staleness * 100.0).round() / 100.0,
            is_stale_for_retest: is_stale,
            advisory,
        });
    }

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_query_failure_vault() {
        let conn = Connection::open_in_memory().unwrap();
        init_failure_vault_table(&conn).unwrap();

        let id = store_vault_failure(
            &conn,
            "laravel-inertia-ssr-window-access",
            "typescript/react",
            "Direct window.innerWidth access during SSR render",
            "ReferenceError: window is not defined",
            "Node.js server-side environment lacks browser window DOM globals",
            "Node 20 / PHP 8.2 / React 18",
            "FATAL_CRASH",
            "Use useEffect or check typeof window !== 'undefined'",
            60,
        ).unwrap();

        assert!(id.starts_with("FAIL-"));

        let dead_ends = query_vault_dead_ends(&conn, "ssr-window-access", 0.8).unwrap();
        assert_eq!(dead_ends.len(), 1);
        assert_eq!(dead_ends[0].fatal_severity, "FATAL_CRASH");
        assert!(!dead_ends[0].is_stale_for_retest);
        assert!(dead_ends[0].advisory.contains("ACTIVE DEAD-END"));
    }
}
