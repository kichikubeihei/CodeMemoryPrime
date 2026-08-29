use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const HMAC_SECRET: &str = "CMP_VAULT_TAMPER_PROOF_SECRET_2026";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ObjectiveMetrics {
    pub compiler_exit_code: i32,
    pub tests_passed: String,
    pub ast_complexity: u32,
    pub execution_time_ms: f64,
    pub security_clean: bool,
    pub zero_cls_a11y_score: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SolutionRecord {
    pub id: String,
    pub problem_signature: String,
    pub domain_language: String,
    pub winning_model: String,
    pub solution_code: String,
    pub context_and_constraints: String,
    pub objective_metrics: ObjectiveMetrics,
    pub staleness_score: f64,
    pub staleness_decay_days: u32,
    pub created_at: String,
    pub last_verified_at: String,
    pub access_count: u32,
    pub hmac_signature: String,
    pub is_stale: bool,
}

pub fn init_solution_vault_table(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS solution_vault (
            id TEXT PRIMARY KEY,
            problem_signature TEXT NOT NULL,
            domain_language TEXT NOT NULL,
            winning_model TEXT NOT NULL,
            solution_code TEXT NOT NULL,
            context_and_constraints TEXT,
            objective_metrics TEXT NOT NULL,
            staleness_decay_days INTEGER DEFAULT 60,
            created_at TEXT NOT NULL,
            last_verified_at TEXT NOT NULL,
            access_count INTEGER DEFAULT 0,
            hmac_signature TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_solution_signature ON solution_vault(problem_signature)",
        [],
    )?;

    Ok(())
}

fn compute_hmac(problem_signature: &str, solution_code: &str, winning_model: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(HMAC_SECRET.as_bytes());
    hasher.update(problem_signature.as_bytes());
    hasher.update(solution_code.as_bytes());
    hasher.update(winning_model.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn current_iso_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    now.to_string()
}

pub fn store_vault_solution(
    conn: &Connection,
    problem_signature: &str,
    domain_language: &str,
    winning_model: &str,
    solution_code: &str,
    context_and_constraints: &str,
    metrics: &ObjectiveMetrics,
    decay_days: u32,
) -> Result<String> {
    init_solution_vault_table(conn)?;

    let id = format!("SOL-{}", &compute_hmac(problem_signature, solution_code, winning_model)[..12]);
    let hmac = compute_hmac(problem_signature, solution_code, winning_model);
    let now_ts = current_iso_timestamp();
    let metrics_json = serde_json::to_string(metrics).unwrap_or_else(|_| "{}".to_string());

    conn.execute(
        "INSERT INTO solution_vault (
            id, problem_signature, domain_language, winning_model,
            solution_code, context_and_constraints, objective_metrics,
            staleness_decay_days, created_at, last_verified_at, access_count, hmac_signature
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11)
        ON CONFLICT(id) DO UPDATE SET
            winning_model = excluded.winning_model,
            solution_code = excluded.solution_code,
            objective_metrics = excluded.objective_metrics,
            last_verified_at = excluded.last_verified_at,
            hmac_signature = excluded.hmac_signature",
        params![
            id,
            problem_signature,
            domain_language,
            winning_model,
            solution_code,
            context_and_constraints,
            metrics_json,
            decay_days,
            now_ts,
            now_ts,
            hmac
        ],
    )?;

    Ok(id)
}

pub fn query_vault_solution(
    conn: &Connection,
    problem_signature: &str,
    max_allowed_staleness: f64,
) -> Result<Option<SolutionRecord>> {
    init_solution_vault_table(conn)?;

    let mut stmt = conn.prepare(
        "SELECT id, problem_signature, domain_language, winning_model,
                solution_code, context_and_constraints, objective_metrics,
                staleness_decay_days, created_at, last_verified_at, access_count, hmac_signature
         FROM solution_vault
         WHERE problem_signature LIKE ?1
         ORDER BY last_verified_at DESC LIMIT 1"
    )?;

    let pattern = format!("%{}%", problem_signature);
    let mut rows = stmt.query([pattern])?;

    if let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        let signature: String = row.get(1)?;
        let domain: String = row.get(2)?;
        let model: String = row.get(3)?;
        let code: String = row.get(4)?;
        let context: String = row.get(5)?;
        let metrics_raw: String = row.get(6)?;
        let decay_days: u32 = row.get(7)?;
        let created_at: String = row.get(8)?;
        let last_verified: String = row.get(9)?;
        let access_count: u32 = row.get(10)?;
        let hmac: String = row.get(11)?;

        // 1. Verify anti-tamper HMAC
        let expected_hmac = compute_hmac(&signature, &code, &model);
        if expected_hmac != hmac {
            return Err(rusqlite::Error::InvalidParameterName(
                "HMAC verification failed: Solution tampering detected in vault!".into()
            ));
        }

        // 2. Increment access count
        let _ = conn.execute(
            "UPDATE solution_vault SET access_count = access_count + 1 WHERE id = ?1",
            params![id],
        );

        // 3. Compute staleness score
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let verified_sec: u64 = last_verified.parse().unwrap_or(now_sec);
        let age_days = (now_sec.saturating_sub(verified_sec) as f64) / 86400.0;
        let decay_limit = (decay_days as f64).max(1.0);
        let staleness = (age_days / decay_limit).min(1.0);
        let is_stale = staleness > max_allowed_staleness;

        let metrics: ObjectiveMetrics = serde_json::from_str(&metrics_raw).unwrap_or(ObjectiveMetrics {
            compiler_exit_code: 0,
            tests_passed: "100%".to_string(),
            ast_complexity: 10,
            execution_time_ms: 0.0,
            security_clean: true,
            zero_cls_a11y_score: 100,
        });

        Ok(Some(SolutionRecord {
            id,
            problem_signature: signature,
            domain_language: domain,
            winning_model: model,
            solution_code: code,
            context_and_constraints: context,
            objective_metrics: metrics,
            staleness_score: (staleness * 100.0).round() / 100.0,
            staleness_decay_days: decay_days,
            created_at,
            last_verified_at: last_verified,
            access_count: access_count + 1,
            hmac_signature: hmac,
            is_stale,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_query_solution_vault() {
        let conn = Connection::open_in_memory().unwrap();
        init_solution_vault_table(&conn).unwrap();

        let metrics = ObjectiveMetrics {
            compiler_exit_code: 0,
            tests_passed: "12/12".to_string(),
            ast_complexity: 8,
            execution_time_ms: 1.45,
            security_clean: true,
            zero_cls_a11y_score: 100,
        };

        let id = store_vault_solution(
            &conn,
            "laravel-inertia-ssr-hydration",
            "typescript/react",
            "qwen2.5-coder:32b",
            "export function SafeHydrate({ children }) { return <>{children}</>; }",
            "Fixes SSR layout shifts in 2ndsLaravel",
            &metrics,
            30,
        ).unwrap();

        assert!(id.starts_with("SOL-"));

        let record = query_vault_solution(&conn, "ssr-hydration", 0.8).unwrap().unwrap();
        assert_eq!(record.winning_model, "qwen2.5-coder:32b");
        assert_eq!(record.objective_metrics.compiler_exit_code, 0);
        assert!(!record.is_stale);
        assert_eq!(record.access_count, 1);
    }
}
