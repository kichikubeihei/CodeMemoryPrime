use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::{llm, license, get_db_path};

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "project_health",
            "description": "Performs system health check: DB location & size, Ollama connection status, BSL 1.1 license status, and indexed project stats.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "summarize_project",
            "description": "Generates a structured architecture & project brief for an indexed project.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_name": { "type": "string", "description": "Project identifier." }
                },
                "required": ["project_name"]
            }
        }),
        json!({
            "name": "configure_settings",
            "description": "Manages LLM provider, base URL, models, and license configuration.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "description": "'get', 'set', or 'reset'." },
                    "provider": { "type": "string", "description": "'ollama' or 'openai'." },
                    "base_url": { "type": "string", "description": "LLM endpoint URL." },
                    "gen_model": { "type": "string", "description": "Generation model name." },
                    "embed_model": { "type": "string", "description": "Embedding model name." },
                    "api_key": { "type": "string", "description": "API key." },
                    "use_framework_grounding": { "type": "boolean", "description": "Use RAG framework specs for code generation." },
                    "framework_grounding_chunks": { "type": "integer", "description": "Number of RAG chunks to include (max 10)." }
                }
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    match name {
        "project_health" => {
            let db_path = get_db_path();
            let db_metadata = std::fs::metadata(&db_path);
            let db_size_mb = db_metadata.map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0);

            let ollama_status = rt.block_on(async {
                let cfg = llm::get_config_from_db_or_env();
                match llm::check_ollama_connection().await {
                    Ok(_) => format!("Connected ({})", cfg.base_url),
                    Err(e) => format!("Disconnected (Error: {})", e),
                }
            });

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let projects_summary: Vec<String> = conn.prepare(
                    "SELECT project_name, COUNT(DISTINCT file_path), COUNT(*) FROM code_chunks GROUP BY project_name"
                ).map(|mut stmt| {
                    stmt.query_map([], |row| {
                        let proj: String = row.get(0)?;
                        let files: i64 = row.get(1)?;
                        let chunks: i64 = row.get(2)?;
                        Ok(format!("  - **{}**: {} files, {} code chunks", proj, files, chunks))
                    }).unwrap().flatten().collect()
                }).unwrap_or_default();

                let uncons_count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM journal_entries WHERE consolidated = 0", [], |r| r.get(0)
                ).unwrap_or(0);

                let framework_docs_count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM framework_documentation", [], |r| r.get(0)
                ).unwrap_or(0);

                let lic_status = license::check_license_key(None);
                let lic_str = match lic_status {
                    license::LicenseStatus::FreeTier { message } => message,
                    license::LicenseStatus::ValidCommercial { licensee, seats, expires, license_type } => {
                        format!("Valid {} License ({}) - {} seats (Expires: {})", license_type, licensee, seats, expires)
                    },
                    license::LicenseStatus::Expired { licensee, expires } => {
                        format!("Expired Commercial License for {} (Expired on: {})", licensee, expires)
                    },
                    license::LicenseStatus::Invalid { reason } => format!("Invalid License Key: {}", reason),
                };

                let llm_auto_summary = rt.block_on(async { llm::auto_detect_llm_setup().await });

                Some(format!(
                    "=== CodeMemoryPrime (CMP) Health Report ===\n\n- **License Status**: {}\n- **Database Location**: `{}` ({:.2} MB)\n- **Ollama Status**: {}\n- **Indexed Framework Docs**: {}\n- **Unconsolidated Journal Entries**: {}\n\n### Indexed Projects:\n{}\n\n{}",
                    lic_str, db_path, db_size_mb, ollama_status, framework_docs_count, uncons_count,
                    if projects_summary.is_empty() { "  (No projects indexed yet. Run index_workspace to index a project.)".to_string() } else { projects_summary.join("\n") },
                    llm_auto_summary
                ))
            } else {
                Some(format!("Database Error: Unable to open database at '{}'.", db_path))
            }
        }
        "summarize_project" => {
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("");
            if project.is_empty() {
                Some("Error: 'project_name' is required.".to_string())
            } else {
                let db_path = get_db_path();
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    let file_summary: Vec<String> = conn.prepare(
                        "SELECT DISTINCT file_name, chunk_type, summary FROM code_chunks WHERE project_name = ?1 LIMIT 25"
                    ).map(|mut stmt| {
                        stmt.query_map(rusqlite::params![project], |row| {
                            let fn_str: String = row.get(0)?;
                            let ct_str: String = row.get(1)?;
                            let s_str: String = row.get(2)?;
                            Ok(format!("- `{}` ({}): {}", fn_str, ct_str, s_str))
                        }).unwrap().flatten().collect()
                    }).unwrap_or_default();

                    let recent_journals: Vec<String> = conn.prepare(
                        "SELECT user_request, ai_response FROM journal_entries WHERE project_name = ?1 ORDER BY entry_date DESC LIMIT 5"
                    ).map(|mut stmt| {
                        stmt.query_map(rusqlite::params![project], |row| {
                            let req: String = row.get(0)?;
                            let resp: String = row.get(1)?;
                            Ok(format!("- Request: {}\n  Summary: {}", req, resp.chars().take(150).collect::<String>()))
                        }).unwrap().flatten().collect()
                    }).unwrap_or_default();

                    let prompt = format!(
                        "Synthesize a clear, structured Project Brief for project '{}'.\n\nTop Code Modules:\n{}\n\nRecent Memory Notes:\n{}\n\nProvide:\n1. Project Overview\n2. Key Modules & Architecture\n3. Recent Progress / Context",
                        project, file_summary.join("\n"), recent_journals.join("\n")
                    );

                    let brief = rt.block_on(async {
                        llm::query_ollama(&prompt).await.unwrap_or_else(|e| format!("LLM query failed: {}", e))
                    });

                    Some(format!("=== Project Brief: {} ===\n\n{}", project, brief))
                } else {
                    Some("Failed to connect to memory database.".to_string())
                }
            }
        }
        "configure_settings" => {
            let action = params.get("action").and_then(|s| s.as_str()).unwrap_or("get");

            match action {
                "set" => {
                    let mut current = llm::get_config_from_db_or_env();
                    if let Some(p) = params.get("provider").and_then(|s| s.as_str()) { current.provider = p.to_string(); }
                    if let Some(b) = params.get("base_url").and_then(|s| s.as_str()) { current.base_url = b.to_string(); }
                    if let Some(g) = params.get("gen_model").and_then(|s| s.as_str()) { current.gen_model = g.to_string(); }
                    if let Some(e) = params.get("embed_model").and_then(|s| s.as_str()) { current.embed_model = e.to_string(); }
                    if let Some(k) = params.get("api_key").and_then(|s| s.as_str()) { current.api_key = k.to_string(); }
                    if let Some(b) = params.get("use_framework_grounding").and_then(|s| s.as_bool()) { current.use_framework_grounding = b; }
                    if let Some(c) = params.get("framework_grounding_chunks").and_then(|s| s.as_u64()) { 
                        current.framework_grounding_chunks = std::cmp::min(c as usize, 10); 
                    }

                    match llm::save_config_to_db(&current) {
                        Ok(_) => {
                            let auto_info = rt.block_on(async { llm::auto_detect_llm_setup().await });
                            Some(format!("=== LLM Settings Updated ===\n- Provider: {}\n- Base URL: {}\n- Generation Model: {}\n- Embedding Model: {}\n- API Key: {}\n- RAG Grounding: {} ({} chunks)\n\n{}",
                                current.provider, current.base_url, current.gen_model, current.embed_model,
                                if current.api_key.is_empty() { "(None)" } else { "***set***" }, current.use_framework_grounding, current.framework_grounding_chunks, auto_info))
                        }
                        Err(err) => Some(format!("Error saving settings to database: {}", err)),
                    }
                }
                "reset" => {
                    let default_cfg = llm::LlmConfig::default();
                    match llm::save_config_to_db(&default_cfg) {
                        Ok(_) => {
                            let auto_info = rt.block_on(async { llm::auto_detect_llm_setup().await });
                            Some(format!("LLM settings reset to defaults (Ollama at http://127.0.0.1:11434 with qwen2.5-coder:7b & nomic-embed-text).\n\n{}", auto_info))
                        }
                        Err(err) => Some(format!("Error resetting settings: {}", err)),
                    }
                }
                _ => {
                    let auto_info = rt.block_on(async { llm::auto_detect_llm_setup().await });
                    Some(auto_info)
                }
            }
        }
        _ => None,
    }
}
