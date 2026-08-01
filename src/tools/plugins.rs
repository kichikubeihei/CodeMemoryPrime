use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::{llm, get_db_path};

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "modularize_code",
            "description": "Analyzes code module structures and proposes clean microservice / plugin refactoring boundaries.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Source code." }
                },
                "required": ["code_content"]
            }
        }),
        json!({
            "name": "extract_plugin",
            "description": "Generates a standalone plugin specification and code block.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Original code." },
                    "plugin_name": { "type": "string", "description": "Plugin name." }
                },
                "required": ["code_content", "plugin_name"]
            }
        }),
        json!({
            "name": "publish_plugin",
            "description": "Publishes a plugin definition into the local catalog FTS database.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "plugin_name": { "type": "string", "description": "Plugin name." },
                    "description": { "type": "string", "description": "Description." },
                    "io_specifications": { "type": "string", "description": "I/O Spec." }
                },
                "required": ["plugin_name", "description"]
            }
        }),
        json!({
            "name": "recommend_plugins",
            "description": "Recommends plugins from catalog based on project requirements.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_requirements": { "type": "string", "description": "Requirements." }
                },
                "required": ["project_requirements"]
            }
        }),
        json!({
            "name": "log_token_usage",
            "description": "Logs token savings and performance analytics into memory SQLite.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_name": { "type": "string", "description": "Project identifier." },
                    "tokens_used": { "type": "integer", "description": "Tokens used." },
                    "tokens_without_memory": { "type": "integer", "description": "Tokens without memory." }
                },
                "required": ["project_name", "tokens_used", "tokens_without_memory"]
            }
        }),
        json!({
            "name": "get_token_analytics",
            "description": "Retrieves total token savings and efficiency stats for a project.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_name": { "type": "string", "description": "Project identifier." }
                },
                "required": ["project_name"]
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    match name {
        "modularize_code" => {
            let code = params.get("code_content").and_then(|s| s.as_str()).unwrap_or("");
            let prompt = format!("Propose a modular refactoring plan splitting this monolithic code into decoupled modules:\n\n```\n{}\n```", code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "extract_plugin" => {
            let code = params.get("code_content").and_then(|s| s.as_str()).unwrap_or("");
            let pname = params.get("plugin_name").and_then(|s| s.as_str()).unwrap_or("plugin");
            let prompt = format!("Extract a standalone plugin named '{}' from code:\n\n```\n{}\n```", pname, code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "publish_plugin" => {
            let pname = params.get("plugin_name").and_then(|s| s.as_str()).unwrap_or("");
            let desc = params.get("description").and_then(|s| s.as_str()).unwrap_or("");
            let io_spec = params.get("io_specifications").and_then(|s| s.as_str()).unwrap_or("");

            let db_path = get_db_path();
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let id = uuid::Uuid::new_v4().to_string();
                let content_to_embed = format!("Plugin: {}\nDescription: {}\nIO: {}", pname, desc, io_spec);
                let emb = rt.block_on(async { llm::generate_embedding(&content_to_embed).await.unwrap_or_default() });
                let blob = crate::db::vector_to_blob(&emb);

                let _ = conn.execute(
                    "INSERT INTO plugin_catalog (id, plugin_name, description, io_specifications, embedding) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![id, pname, desc, io_spec, blob]
                );
                let _ = conn.execute(
                    "INSERT INTO plugin_catalog_fts (id, plugin_name, description, io_specifications) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![id, pname, desc, io_spec]
                );
                Some(format!("Published plugin '{}' to local catalog.", pname))
            } else {
                Some("Failed to open database.".to_string())
            }
        }
        "recommend_plugins" => {
            let reqs = params.get("project_requirements").and_then(|s| s.as_str()).unwrap_or("");
            let db_path = get_db_path();

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let mut stmt = conn.prepare("SELECT plugin_name, description, io_specifications FROM plugin_catalog_fts WHERE plugin_catalog_fts MATCH ?1 LIMIT 5").unwrap();
                let mut rows = stmt.query(rusqlite::params![reqs]).unwrap();
                let mut out = format!("Recommended plugins for requirements '{}':\n\n", reqs);
                let mut found = false;
                while let Ok(Some(row)) = rows.next() {
                    found = true;
                    let n: String = row.get(0).unwrap_or_default();
                    let d: String = row.get(1).unwrap_or_default();
                    out.push_str(&format!("- **{}**: {}\n", n, d));
                }
                if !found {
                    out.push_str("No matching plugins found in catalog.");
                }
                Some(out)
            } else {
                Some("Failed to open database.".to_string())
            }
        }
        "log_token_usage" | "dev_log_token_usage" => {
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("");
            let used = params.get("tokens_used").and_then(|s| s.as_u64()).unwrap_or(0);
            let wo_mem = params.get("tokens_without_memory").and_then(|s| s.as_u64()).unwrap_or(0);
            let savings = wo_mem.saturating_sub(used);
            let accuracy = params.get("accuracy_notes").and_then(|s| s.as_str()).unwrap_or("");

            let db_path = get_db_path();
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let id = uuid::Uuid::new_v4().to_string();
                let today = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                let _ = conn.execute(
                    "INSERT INTO token_analytics (id, project_name, tokens_used, tokens_without_memory, token_savings, accuracy_notes, timestamp) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    rusqlite::params![id, project, used as i64, wo_mem as i64, savings as i64, accuracy, today]
                );
                Some(format!("Logged token analytics for project '{}': Saved {} tokens (Used {} vs {} full context).", project, savings, used, wo_mem))
            } else {
                Some("Failed to open database.".to_string())
            }
        }
        "get_token_analytics" | "dev_get_analytics" => {
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("");
            let db_path = get_db_path();

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let mut stmt = conn.prepare("SELECT SUM(tokens_used), SUM(tokens_without_memory), SUM(token_savings) FROM token_analytics WHERE project_name = ?1").unwrap();
                let mut rows = stmt.query(rusqlite::params![project]).unwrap();
                if let Ok(Some(row)) = rows.next() {
                    let u: i64 = row.get(0).unwrap_or(0);
                    let w: i64 = row.get(1).unwrap_or(0);
                    let s: i64 = row.get(2).unwrap_or(0);
                    Some(format!("=== Token Analytics for Project '{}' ===\n- Total Tokens Used: {}\n- Estimated Full Context Tokens: {}\n- Total Token Savings: {} tokens ({:.1}% token reduction)",
                        project, u, w, s, if w > 0 { (s as f64 / w as f64) * 100.0 } else { 0.0 }))
                } else {
                    Some(format!("No analytics recorded for project '{}'.", project))
                }
            } else {
                Some("Failed to open database.".to_string())
            }
        }
        _ => None,
    }
}
