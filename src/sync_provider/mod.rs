pub mod cloudflare_r2;
pub mod tailscale;

use crate::mesh_sync::{export_memory_delta, import_memory_delta};
use cloudflare_r2::CloudflareR2Backend;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use tailscale::TailscaleBackend;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub provider: String, // "r2", "tailscale", "hybrid", "local"
    pub r2_account_id: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub r2_bucket: String,
    pub r2_endpoint: Option<String>,
    pub tailscale_endpoint: String,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: "r2".to_string(),
            r2_account_id: String::new(),
            r2_access_key_id: String::new(),
            r2_secret_access_key: String::new(),
            r2_bucket: "codememory-mesh".to_string(),
            r2_endpoint: None,
            tailscale_endpoint: "http://100.102.233.128:7788".to_string(),
        }
    }
}

pub fn init_settings_table(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS system_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    let _ = init_settings_table(conn);
    let mut stmt = conn.prepare("SELECT value FROM system_settings WHERE key = ?1").ok()?;
    stmt.query_row([key], |r| r.get(0)).ok()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    init_settings_table(conn)?;
    conn.execute(
        "INSERT INTO system_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

pub fn load_sync_config(conn: &Connection) -> SyncConfig {
    let mut config = SyncConfig::default();

    // 1. Check SQLite DB settings
    if let Some(p) = get_setting(conn, "sync_provider") {
        config.provider = p;
    }
    if let Some(v) = get_setting(conn, "r2_account_id") {
        config.r2_account_id = v;
    }
    if let Some(v) = get_setting(conn, "r2_access_key_id") {
        config.r2_access_key_id = v;
    }
    if let Some(v) = get_setting(conn, "r2_secret_access_key") {
        config.r2_secret_access_key = v;
    }
    if let Some(v) = get_setting(conn, "r2_bucket") {
        config.r2_bucket = v;
    }
    if let Some(v) = get_setting(conn, "r2_endpoint") {
        config.r2_endpoint = Some(v);
    }
    if let Some(v) = get_setting(conn, "tailscale_endpoint") {
        config.tailscale_endpoint = v;
    }

    // 2. Override with Environment Variables if present
    if let Ok(p) = std::env::var("MCP_SYNC_PROVIDER").or_else(|_| std::env::var("CMP_SYNC_PROVIDER")) {
        config.provider = p;
    }
    if let Ok(v) = std::env::var("R2_ACCOUNT_ID") {
        config.r2_account_id = v;
    }
    if let Ok(v) = std::env::var("R2_ACCESS_KEY_ID") {
        config.r2_access_key_id = v;
    }
    if let Ok(v) = std::env::var("R2_SECRET_ACCESS_KEY") {
        config.r2_secret_access_key = v;
    }
    if let Ok(v) = std::env::var("R2_BUCKET") {
        config.r2_bucket = v;
    }
    if let Ok(v) = std::env::var("R2_ENDPOINT") {
        config.r2_endpoint = Some(v);
    }
    if let Ok(v) = std::env::var("CMP_TAILSCALE_SYNC_ENDPOINT") {
        config.tailscale_endpoint = v;
    }

    config
}

pub fn save_sync_config(conn: &Connection, config: &SyncConfig) -> Result<(), rusqlite::Error> {
    set_setting(conn, "sync_provider", &config.provider)?;
    if !config.r2_account_id.is_empty() {
        set_setting(conn, "r2_account_id", &config.r2_account_id)?;
    }
    if !config.r2_access_key_id.is_empty() {
        set_setting(conn, "r2_access_key_id", &config.r2_access_key_id)?;
    }
    if !config.r2_secret_access_key.is_empty() {
        set_setting(conn, "r2_secret_access_key", &config.r2_secret_access_key)?;
    }
    if !config.r2_bucket.is_empty() {
        set_setting(conn, "r2_bucket", &config.r2_bucket)?;
    }
    if let Some(ref ep) = config.r2_endpoint {
        set_setting(conn, "r2_endpoint", ep)?;
    }
    if !config.tailscale_endpoint.is_empty() {
        set_setting(conn, "tailscale_endpoint", &config.tailscale_endpoint)?;
    }
    Ok(())
}

pub fn sync_memory_mesh(
    conn: &Connection,
    device_name: &str,
    rt: &tokio::runtime::Runtime,
) -> Result<String, String> {
    let config = load_sync_config(conn);
    let mut log = Vec::new();
    log.push(format!("=== CodeMemoryPrime Distributed Memory Mesh Sync ({}) ===", config.provider.to_uppercase()));

    // 1. Export local delta package
    let local_pkg = export_memory_delta(conn, device_name)
        .map_err(|e| format!("Failed to export local delta: {}", e))?;

    let short_sig = if local_pkg.hmac_signature.len() >= 12 {
        &local_pkg.hmac_signature[..12]
    } else {
        &local_pkg.hmac_signature
    };
    log.push(format!(" [✔] Exported local delta from '{}' (HMAC: `{}`):", device_name, short_sig));
    log.push(format!("     • Session Handoffs: {}", local_pkg.session_handoffs.len()));
    log.push(format!("     • Research / Media Vault: {}", local_pkg.research_vault.len()));
    log.push(format!("     • Solution Vault Records: {}", local_pkg.solution_vault.len()));
    log.push(format!("     • Failure Dead Ends: {}", local_pkg.failure_vault.len()));
    log.push(format!("     • Knowledge Graph Nodes: {}", local_pkg.knowledge_nodes.len()));

    let provider = config.provider.to_lowercase();

    // 2. Perform sync depending on selected provider
    if provider == "r2" || provider == "hybrid" {
        if config.r2_account_id.is_empty() || config.r2_access_key_id.is_empty() || config.r2_secret_access_key.is_empty() {
            if provider == "r2" {
                log.push("\n [!] Cloudflare R2 Credentials Not Fully Configured:".to_string());
                log.push("     Set via CMP tool `configure_sync` or environment variables:".to_string());
                log.push("     • R2_ACCOUNT_ID".to_string());
                log.push("     • R2_ACCESS_KEY_ID".to_string());
                log.push("     • R2_SECRET_ACCESS_KEY".to_string());
                log.push(format!("     • R2_BUCKET (Current: '{}')", config.r2_bucket));
                log.push("     Local delta package remains safely cached and ready to push.".to_string());
                return Ok(log.join("\n"));
            }
        } else {
            let r2 = CloudflareR2Backend::new(
                config.r2_account_id.clone(),
                config.r2_access_key_id.clone(),
                config.r2_secret_access_key.clone(),
                config.r2_bucket.clone(),
                config.r2_endpoint.clone(),
            );

            log.push(format!("\n [i] Connecting to Cloudflare R2 (Bucket: `{}`)...", config.r2_bucket));

            // Push
            match rt.block_on(r2.push_delta_package(&local_pkg)) {
                Ok(remote_key) => {
                    log.push(format!(" [✔] Pushed local delta to R2: `{}`", remote_key));
                }
                Err(e) => {
                    log.push(format!(" [⚠] Push to R2 failed: {}", e));
                }
            }

            // Pull & Merge
            match rt.block_on(r2.pull_all_deltas()) {
                Ok(deltas) => {
                    let mut total_merged_handoffs = 0;
                    let mut total_merged_research = 0;
                    let mut total_merged_solutions = 0;
                    let mut total_merged_nodes = 0;

                    for d in &deltas {
                        // Skip our own package if nothing new
                        if d.device_source == device_name && d.hmac_signature == local_pkg.hmac_signature {
                            continue;
                        }
                        if let Ok(report) = import_memory_delta(conn, d) {
                            total_merged_handoffs += report.handoffs_merged;
                            total_merged_research += report.research_merged;
                            total_merged_solutions += report.solutions_merged;
                            total_merged_nodes += report.nodes_merged;
                        }
                    }

                    log.push(format!(" [✔] Pulled & merged {} remote delta packages from R2:", deltas.len()));
                    log.push(format!("     • Merged Handoffs: {}", total_merged_handoffs));
                    log.push(format!("     • Merged Research / Videos: {}", total_merged_research));
                    log.push(format!("     • Merged Solutions: {}", total_merged_solutions));
                    log.push(format!("     • Merged Knowledge Nodes: {}", total_merged_nodes));
                }
                Err(e) => {
                    log.push(format!(" [⚠] Pull from R2 failed: {}", e));
                }
            }
        }
    }

    if provider == "tailscale" || (provider == "hybrid" && config.r2_account_id.is_empty()) {
        let ts = TailscaleBackend::new(config.tailscale_endpoint.clone());
        log.push(format!("\n [i] Connecting to Tailscale Peer (`{}`)...", config.tailscale_endpoint));

        match rt.block_on(ts.test_connection()) {
            Ok(true) => {
                log.push(" [✔] Tailscale peer is online and responding.".to_string());
                // Push
                if let Err(e) = rt.block_on(ts.push_delta_package(&local_pkg)) {
                    log.push(format!(" [⚠] Tailscale push failed: {}", e));
                }
                // Pull
                if let Ok(deltas) = rt.block_on(ts.pull_all_deltas()) {
                    let mut total_merged = 0;
                    for d in &deltas {
                        if d.device_source != device_name {
                            if let Ok(rep) = import_memory_delta(conn, d) {
                                total_merged += rep.handoffs_merged + rep.research_merged;
                            }
                        }
                    }
                    log.push(format!(" [✔] Merged {} updates from Tailscale peer.", total_merged));
                }
            }
            _ => {
                log.push(format!(" [⚠] Tailscale peer at `{}` is currently offline or daemon not running.", config.tailscale_endpoint));
                log.push("     (Run `cmp serve` on that node to activate Tailscale daemon)".to_string());
            }
        }
    }

    log.push("\n=== Mesh Synchronization Complete ===".to_string());
    Ok(log.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_load_and_save() {
        let conn = Connection::open_in_memory().unwrap();
        let config = SyncConfig {
            provider: "r2".to_string(),
            r2_account_id: "test_acc_123".to_string(),
            r2_access_key_id: "test_key_456".to_string(),
            r2_secret_access_key: "test_sec_789".to_string(),
            r2_bucket: "my-memory-bucket".to_string(),
            r2_endpoint: Some("https://r2.custom.com".to_string()),
            tailscale_endpoint: "http://100.102.233.128:7788".to_string(),
        };

        save_sync_config(&conn, &config).unwrap();
        let loaded = load_sync_config(&conn);

        assert_eq!(loaded.provider, "r2");
        assert_eq!(loaded.r2_account_id, "test_acc_123");
        assert_eq!(loaded.r2_access_key_id, "test_key_456");
        assert_eq!(loaded.r2_bucket, "my-memory-bucket");
        assert_eq!(loaded.r2_endpoint, Some("https://r2.custom.com".to_string()));
    }
}
