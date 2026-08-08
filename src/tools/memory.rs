use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::{db, llm, get_db_path};

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "save_interaction",
            "description": "Saves developer preferences, design decisions, user requests, or AI response summaries into the persistent memory journal. Call this whenever the user expresses explicit workflow preferences, architectural rules, or key decisions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "user_request": { "type": "string", "description": "The prompt or user decision to remember." },
                    "ai_response": { "type": "string", "description": "The AI response or key decision summary to record." },
                    "project_name": { "type": "string", "description": "Project identifier." }
                },
                "required": ["user_request", "ai_response", "project_name"]
            }
        }),
        json!({
            "name": "search_memories",
            "description": "Searches consolidated facts, architectural decisions, and journal history stored in the persistent memory database. Call this when starting a task to recall prior user instructions, architectural preferences, or past solutions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Topic or keyword to recall (e.g. 'auth strategy', 'database schema')." },
                    "project_name": { "type": "string", "description": "Project name or 'all'." },
                    "limit": { "type": "integer", "description": "Max results to return (default 3)." }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "consolidate_memories",
            "description": "Runs LLM consolidation over raw interaction journals, distilling them into structured permanent facts and architectural rules stored in SQLite. Keeps memory compact and fast.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_name": { "type": "string", "description": "Project identifier to consolidate." }
                },
                "required": ["project_name"]
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    match name {
        "save_interaction" => {
            let req = params.get("user_request").and_then(|s| s.as_str()).unwrap_or("");
            let ai_resp = params.get("ai_response").and_then(|s| s.as_str()).unwrap_or("");
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("");

            let db_path = get_db_path();
            let _ = db::init_database(&db_path);

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let entry_id = uuid::Uuid::new_v4().to_string();
                let today = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

                let _ = conn.execute(
                    "INSERT INTO journal_entries (id, user_request, ai_response, project_name, entry_date) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![entry_id, req, ai_resp, project, today]
                );

                Some(format!("Interaction saved to persistent memory for project '{}'.", project))
            } else {
                Some("Failed to open database.".to_string())
            }
        }
        "search_memories" => {
            let query = params.get("query").and_then(|s| s.as_str()).unwrap_or("");
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let limit = params.get("limit").and_then(|s| s.as_u64()).unwrap_or(3) as usize;

            let db_path = get_db_path();
            let _ = db::init_database(&db_path);

            let query_emb = rt.block_on(async {
                llm::generate_embedding(query).await.unwrap_or_default()
            });

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let facts = crate::search::query_consolidated_facts(&conn, query, &query_emb, project, limit).unwrap_or_default();

                let mut out = format!("=== Memory Search Results for '{}' ===\n\n", query);
                if facts.is_empty() {
                    if let Ok(mut stmt) = conn.prepare("SELECT user_request, ai_response, entry_date FROM journal_entries WHERE project_name = ?1 OR ?1 = 'all' ORDER BY entry_date DESC LIMIT ?2") {
                        if let Ok(mut rows) = stmt.query(rusqlite::params![project, limit as i64]) {
                            let mut journal_count = 0;

                            while let Ok(Some(row)) = rows.next() {
                                if journal_count == 0 {
                                    out.push_str("No consolidated facts found. Checking raw journal history...\n\n");
                                }
                                journal_count += 1;
                                let u: String = row.get(0).unwrap_or_default();
                                let a: String = row.get(1).unwrap_or_default();
                                let d: String = row.get(2).unwrap_or_default();
                                out.push_str(&format!("- **[{}]**\n  *User:* {}\n  *Summary:* {}\n\n", d, u, a));
                            }

                            if journal_count == 0 {
                                out.push_str(&format!(
                                    "[Notice] No memory facts or journal entries found for project '{}'.\n\nTo save architectural decisions or user preferences into persistent memory:\nCall tool `save_interaction(user_request='...', ai_response='...', project_name='{}')`.",
                                    project, project
                                ));
                            }
                        }
                    }
                } else {
                    for f in facts {
                        out.push_str(&format!("- **[{}]** (Category: `{}`)\n  {}\n\n", f.fact_type, f.category, f.fact_content));
                    }
                }
                Some(out)
            } else {
                Some("Failed to connect to memory database.".to_string())
            }
        }
        "consolidate_memories" => {
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("");
            let db_path = get_db_path();

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let mut entries = Vec::new();
                if let Ok(mut stmt) = conn.prepare("SELECT id, user_request, ai_response FROM journal_entries WHERE project_name = ?1 AND consolidated = 0 LIMIT 20") {
                    if let Ok(mut rows) = stmt.query(rusqlite::params![project]) {
                        while let Ok(Some(row)) = rows.next() {
                            let id: String = row.get(0).unwrap_or_default();
                            let req: String = row.get(1).unwrap_or_default();
                            let resp: String = row.get(2).unwrap_or_default();
                            entries.push((id, req, resp));
                        }
                    }
                }

                if entries.is_empty() {
                    Some(format!("No new unconsolidated journal entries for project '{}'.", project))
                } else {
                    let mut text_block = String::new();
                    for (_, req, resp) in &entries {
                        text_block.push_str(&format!("User Request: {}\nAI Response: {}\n---\n", req, resp));
                    }

                    let prompt = format!(
                        "Analyze these developer interaction logs for project '{}'. Extract key developer decisions, architectural rules, user preferences, and tech choices. Return a JSON array of objects, each with 'category', 'fact_type' ('decision'|'preference'|'architecture'), and 'content'.\n\nLogs:\n{}",
                        project, text_block
                    );

                    let llm_resp = rt.block_on(async {
                        llm::query_ollama(&prompt).await.unwrap_or_default()
                    });

                    let mut count = 0;
                    if let Ok(json_val) = serde_json::from_str::<Value>(&llm_resp) {
                        if let Some(arr) = json_val.as_array() {
                            for item in arr {
                                let cat = item.get("category").and_then(|s| s.as_str()).unwrap_or("general");
                                let ftype = item.get("fact_type").and_then(|s| s.as_str()).unwrap_or("decision");
                                let content = item.get("content").and_then(|s| s.as_str()).unwrap_or("");

                                if !content.is_empty() {
                                    let emb = rt.block_on(async {
                                        llm::generate_embedding(content).await.unwrap_or_default()
                                    });
                                    let blob = db::vector_to_blob(&emb);
                                    let fact_id = uuid::Uuid::new_v4().to_string();

                                    let _ = conn.execute(
                                        "INSERT INTO consolidated_facts (id, project_name, category, fact_type, fact_content, embedding) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                        rusqlite::params![fact_id, project, cat, ftype, content, blob]
                                    );
                                    count += 1;
                                }
                            }
                        }
                    }

                    for (id, _, _) in &entries {
                        let _ = conn.execute("UPDATE journal_entries SET consolidated = 1 WHERE id = ?1", rusqlite::params![id]);
                    }

                    Some(format!("Consolidated {} raw interactions into {} structured facts for project '{}'.", entries.len(), count, project))
                }
            } else {
                Some("Failed to connect to database.".to_string())
            }
        }
        _ => None,
    }
}
