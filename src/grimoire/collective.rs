use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedSpell {
    pub algorithm_id: String,
    pub language: String,
    pub code_snippet: String,
    pub author_alias: String,
    pub execution_time_us: f64,
    pub contracts_passed: usize,
    pub hmac_signature: String,
    pub harvested_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeVerdict {
    pub strike_count: usize,
    pub is_banned: bool,
    pub status_message: String,
}

pub fn init_grimoire_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS grimoire_collective (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            algorithm_id TEXT NOT NULL,
            language TEXT NOT NULL,
            code_snippet TEXT NOT NULL,
            author_alias TEXT NOT NULL,
            execution_time_us REAL NOT NULL,
            contracts_passed INTEGER NOT NULL,
            hmac_signature TEXT NOT NULL,
            harvested_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cheater_strikes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            client_pubkey TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            strike_count INTEGER NOT NULL DEFAULT 0,
            last_violation TEXT NOT NULL,
            last_strike_at INTEGER NOT NULL,
            is_banned INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_grimoire_algo_lang ON grimoire_collective(algorithm_id, language)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_cheater_ip_key ON cheater_strikes(ip_address, client_pubkey)",
        [],
    )?;

    Ok(())
}

pub fn inscribe_winning_spell(
    conn: &Connection,
    algorithm_id: &str,
    language: &str,
    code: &str,
    author: &str,
    exec_time_us: f64,
    contracts_passed: usize,
) -> Result<VerifiedSpell> {
    init_grimoire_tables(conn)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Compute cryptographic signature proving verification
    let mut hasher = Sha256::new();
    hasher.update(algorithm_id.as_bytes());
    hasher.update(language.as_bytes());
    hasher.update(code.as_bytes());
    hasher.update(exec_time_us.to_le_bytes());
    hasher.update(now.to_le_bytes());
    let sig = hex::encode(hasher.finalize());

    conn.execute(
        "INSERT INTO grimoire_collective 
        (algorithm_id, language, code_snippet, author_alias, execution_time_us, contracts_passed, hmac_signature, harvested_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            algorithm_id,
            language,
            code,
            author,
            exec_time_us,
            contracts_passed as i64,
            sig,
            now as i64
        ],
    )?;

    Ok(VerifiedSpell {
        algorithm_id: algorithm_id.to_string(),
        language: language.to_string(),
        code_snippet: code.to_string(),
        author_alias: author.to_string(),
        execution_time_us: exec_time_us,
        contracts_passed,
        hmac_signature: sig,
        harvested_at: now,
    })
}

pub fn query_collective_spells(
    conn: &Connection,
    algorithm_id: Option<&str>,
    language: Option<&str>,
) -> Result<Vec<VerifiedSpell>> {
    init_grimoire_tables(conn)?;

    let mut query = "SELECT algorithm_id, language, code_snippet, author_alias, execution_time_us, contracts_passed, hmac_signature, harvested_at FROM grimoire_collective WHERE 1=1".to_string();
    if algorithm_id.is_some() {
        query.push_str(" AND algorithm_id = ?1");
    }
    if language.is_some() {
        query.push_str(" AND language = ?2");
    }
    query.push_str(" ORDER BY execution_time_us ASC");

    let mut stmt = conn.prepare(&query)?;
    let mut spells = Vec::new();

    match (algorithm_id, language) {
        (Some(a), Some(l)) => {
            for row in stmt.query_map(params![a, l], read_spell_row)? {
                if let Ok(s) = row { spells.push(s); }
            }
        }
        (Some(a), None) => {
            for row in stmt.query_map(params![a], read_spell_row)? {
                if let Ok(s) = row { spells.push(s); }
            }
        }
        (None, Some(l)) => {
            for row in stmt.query_map(params![l], read_spell_row)? {
                if let Ok(s) = row { spells.push(s); }
            }
        }
        (None, None) => {
            for row in stmt.query_map([], read_spell_row)? {
                if let Ok(s) = row { spells.push(s); }
            }
        }
    }

    Ok(spells)
}

fn read_spell_row(r: &rusqlite::Row) -> rusqlite::Result<VerifiedSpell> {
    Ok(VerifiedSpell {
        algorithm_id: r.get(0)?,
        language: r.get(1)?,
        code_snippet: r.get(2)?,
        author_alias: r.get(3)?,
        execution_time_us: r.get(4)?,
        contracts_passed: r.get::<_, i64>(5)? as usize,
        hmac_signature: r.get(6)?,
        harvested_at: r.get::<_, i64>(7)? as u64,
    })
}

/// Records a strike against an offending IP and Client Public Key (3-Strikes Enforcement)
pub fn record_strike(
    conn: &Connection,
    ip_address: &str,
    client_pubkey: &str,
    reason: &str,
) -> Result<StrikeVerdict> {
    init_grimoire_tables(conn)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let existing: Result<(i64, i64, i64), _> = conn.query_row(
        "SELECT id, strike_count, is_banned FROM cheater_strikes WHERE ip_address = ?1 OR client_pubkey = ?2",
        params![ip_address, client_pubkey],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    );

    let (current_strikes, is_banned) = if let Ok((id, strikes, banned)) = existing {
        let new_strikes = strikes + 1;
        let new_banned = if new_strikes >= 3 { 1 } else { banned };

        conn.execute(
            "UPDATE cheater_strikes SET strike_count = ?1, is_banned = ?2, last_violation = ?3, last_strike_at = ?4 WHERE id = ?5",
            params![new_strikes, new_banned, reason, now as i64, id],
        )?;
        (new_strikes, new_banned)
    } else {
        conn.execute(
            "INSERT INTO cheater_strikes (client_pubkey, ip_address, strike_count, last_violation, last_strike_at, is_banned)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![client_pubkey, ip_address, 1, reason, now as i64, 0],
        )?;
        (1, 0)
    };

    let status_message = if is_banned == 1 {
        format!("STRIKE 3 REACHED: Client {} / IP {} is PERMANENTLY BANNED from Grimoire Gauntlet.", client_pubkey, ip_address)
    } else {
        format!("STRIKE {} RECORDED: {} ({} strikes remaining before automated WAF drop).", current_strikes, reason, 3 - current_strikes)
    };

    Ok(StrikeVerdict {
        strike_count: current_strikes as usize,
        is_banned: is_banned == 1,
        status_message,
    })
}

pub fn check_ban_status(
    conn: &Connection,
    ip_address: &str,
    client_pubkey: &str,
) -> Result<bool> {
    init_grimoire_tables(conn)?;

    let banned: Result<i64, _> = conn.query_row(
        "SELECT is_banned FROM cheater_strikes WHERE (ip_address = ?1 OR client_pubkey = ?2) AND is_banned = 1 LIMIT 1",
        params![ip_address, client_pubkey],
        |r| r.get(0),
    );

    Ok(banned.unwrap_or(0) == 1)
}
