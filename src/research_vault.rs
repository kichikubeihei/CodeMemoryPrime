use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResearchRecord {
    pub id: String,
    pub media_url: String,
    pub title: String,
    pub media_type: String, // "youtube_video", "technical_paper", "web_article", "repo"
    pub target_project: String,
    pub key_takeaways: String,
    pub proposed_upgrades: String,
    pub hmac_signature: String,
    pub created_at: String,
}

pub fn init_research_vault_tables(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS research_vault (
            id TEXT PRIMARY KEY,
            media_url TEXT NOT NULL,
            title TEXT NOT NULL,
            media_type TEXT NOT NULL,
            target_project TEXT NOT NULL,
            key_takeaways TEXT NOT NULL,
            proposed_upgrades TEXT NOT NULL,
            hmac_signature TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_research_vault_project ON research_vault (target_project)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_research_vault_created_at ON research_vault (created_at DESC)",
        [],
    )?;

    Ok(())
}

pub fn compute_research_hmac(id: &str, media_url: &str, title: &str, takeaways: &str, upgrades: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(id.as_bytes());
    hasher.update(media_url.as_bytes());
    hasher.update(title.as_bytes());
    hasher.update(takeaways.as_bytes());
    hasher.update(upgrades.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn record_research(
    conn: &Connection,
    media_url: &str,
    title: &str,
    media_type: &str,
    target_project: &str,
    key_takeaways: &str,
    proposed_upgrades: &str,
) -> Result<ResearchRecord> {
    init_research_vault_tables(conn)?;

    let id = format!("RES-{}", Uuid::new_v4());
    let created_at = chrono::Utc::now().to_rfc3339();
    let hmac_signature = compute_research_hmac(&id, media_url, title, key_takeaways, proposed_upgrades);

    let record = ResearchRecord {
        id: id.clone(),
        media_url: media_url.to_string(),
        title: title.to_string(),
        media_type: media_type.to_string(),
        target_project: target_project.to_string(),
        key_takeaways: key_takeaways.to_string(),
        proposed_upgrades: proposed_upgrades.to_string(),
        hmac_signature: hmac_signature.clone(),
        created_at: created_at.clone(),
    };

    conn.execute(
        "INSERT INTO research_vault (id, media_url, title, media_type, target_project, key_takeaways, proposed_upgrades, hmac_signature, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
             title=excluded.title,
             key_takeaways=excluded.key_takeaways,
             proposed_upgrades=excluded.proposed_upgrades,
             hmac_signature=excluded.hmac_signature",
        params![
            record.id,
            record.media_url,
            record.title,
            record.media_type,
            record.target_project,
            record.key_takeaways,
            record.proposed_upgrades,
            record.hmac_signature,
            record.created_at,
        ],
    )?;

    Ok(record)
}

pub fn query_research(
    conn: &Connection,
    target_project: &str,
    keyword: &str,
    limit: usize,
) -> Result<Vec<ResearchRecord>> {
    init_research_vault_tables(conn)?;

    let mut sql = "SELECT id, media_url, title, media_type, target_project, key_takeaways, proposed_upgrades, hmac_signature, created_at 
                   FROM research_vault".to_string();
    let mut conditions = Vec::new();
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if target_project != "all" && !target_project.trim().is_empty() {
        conditions.push("target_project = ?");
        params_vec.push(Box::new(target_project.to_string()));
    }

    if !keyword.trim().is_empty() {
        conditions.push("(title LIKE '%' || ? || '%' OR key_takeaways LIKE '%' || ? || '%' OR proposed_upgrades LIKE '%' || ? || '%')");
        params_vec.push(Box::new(keyword.to_string()));
        params_vec.push(Box::new(keyword.to_string()));
        params_vec.push(Box::new(keyword.to_string()));
    }

    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT {}", limit.max(1)));

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(ResearchRecord {
            id: row.get(0)?,
            media_url: row.get(1)?,
            title: row.get(2)?,
            media_type: row.get(3)?,
            target_project: row.get(4)?,
            key_takeaways: row.get(5)?,
            proposed_upgrades: row.get(6)?,
            hmac_signature: row.get(7)?,
            created_at: row.get(8)?,
        })
    })?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

pub fn get_latest_research(conn: &Connection, target_project: &str) -> Result<Option<ResearchRecord>> {
    let mut results = query_research(conn, target_project, "", 1)?;
    Ok(results.pop())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_query_research() {
        let conn = Connection::open_in_memory().unwrap();
        init_research_vault_tables(&conn).unwrap();

        let rec = record_research(
            &conn,
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "Advanced Svelte 5 Runes & Ast Performance",
            "youtube_video",
            "CodeMemoryPrime",
            "Runes replace let bindings with $state and $derived signals for 3x lower memory.",
            "Upgrade CodeMemoryPrime AST parser to tokenize runes as native reactive primitives.",
        ).unwrap();

        assert!(rec.id.starts_with("RES-"));
        assert_eq!(rec.media_type, "youtube_video");

        let found = query_research(&conn, "CodeMemoryPrime", "Runes", 5).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].title, "Advanced Svelte 5 Runes & Ast Performance");

        let latest = get_latest_research(&conn, "all").unwrap();
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().media_url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }
}
