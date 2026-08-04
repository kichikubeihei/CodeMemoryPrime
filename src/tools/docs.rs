use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::{db, llm, scraper, get_db_path};

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "index_framework_specifications",
            "description": "Indexes framework documentation from local markdown files or web URLs into vector memory.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "framework_name": { "type": "string", "description": "Framework name (e.g. 'tokio', 'nextjs')." },
                    "category": { "type": "string", "description": "Category tag." },
                    "version": { "type": "string", "description": "Version string." },
                    "source_path_or_url": { "type": "string", "description": "File folder or web URL." }
                },
                "required": ["framework_name", "source_path_or_url"]
            }
        }),
        json!({
            "name": "search_framework_specifications",
            "description": "Hybrid vector search across indexed framework documentation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query." },
                    "category": { "type": "string", "description": "Category filter." },
                    "limit": { "type": "integer", "description": "Max results (default 5)." }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "generate_documentation",
            "description": "Generates docstrings and API documentation for code blocks.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Source code." }
                },
                "required": ["code_content"]
            }
        }),
        json!({
            "name": "get_documentation",
            "description": "Retrieves documentation comments for functions or structs.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "symbol_name": { "type": "string", "description": "Symbol to inspect." }
                },
                "required": ["symbol_name"]
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    match name {
        "index_framework_specifications" => {
            let fw = params.get("framework_name").and_then(|s| s.as_str()).unwrap_or("");
            let category = params.get("category").and_then(|s| s.as_str()).unwrap_or("");
            let version = params.get("version").and_then(|s| s.as_str()).unwrap_or("latest");
            let source = params.get("source_path_or_url").and_then(|s| s.as_str()).unwrap_or("");

            let db_path = get_db_path();
            let _ = db::init_database(&db_path);

            if source.starts_with("http://") || source.starts_with("https://") {
                let blocks = rt.block_on(async { scraper::scrape_web_docs(source).await });
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    for block in &blocks {
                        let emb = rt.block_on(async { llm::generate_embedding(&block.content).await.unwrap_or_default() });
                        let blob = db::vector_to_blob(&emb);
                        let id = uuid::Uuid::new_v4().to_string();
                        let _ = conn.execute(
                            "INSERT INTO framework_documentation (id, category, version, title, url, content, embedding) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![id, category, version, block.title, block.url, block.content, blob]
                        );
                        let _ = conn.execute(
                            "INSERT INTO framework_documentation_fts (id, category, version, title, url, content) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            rusqlite::params![id, category, version, block.title, block.url, block.content]
                        );
                        for (link_text, target_url) in &block.links {
                            let link_id = uuid::Uuid::new_v4().to_string();
                            let _ = conn.execute(
                                "INSERT INTO framework_dependencies (id, source_url, target_url, link_text) VALUES (?1, ?2, ?3, ?4)",
                                rusqlite::params![link_id, block.url, target_url, link_text]
                            );
                        }
                    }
                    Some(format!("Indexed {} documentation sections from URL '{}' for framework '{}'.", blocks.len(), source, fw))
                } else {
                    Some("Failed to open database.".to_string())
                }
            } else {
                let blocks = scraper::parse_local_markdown_docs(source);
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    for block in &blocks {
                        let emb = rt.block_on(async { llm::generate_embedding(&block.content).await.unwrap_or_default() });
                        let blob = db::vector_to_blob(&emb);
                        let id = uuid::Uuid::new_v4().to_string();
                        let _ = conn.execute(
                            "INSERT INTO framework_documentation (id, category, version, title, url, content, embedding) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![id, category, version, block.title, block.url, block.content, blob]
                        );
                        let _ = conn.execute(
                            "INSERT INTO framework_documentation_fts (id, category, version, title, url, content) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            rusqlite::params![id, category, version, block.title, block.url, block.content]
                        );
                        for (link_text, target_url) in &block.links {
                            let link_id = uuid::Uuid::new_v4().to_string();
                            let _ = conn.execute(
                                "INSERT INTO framework_dependencies (id, source_url, target_url, link_text) VALUES (?1, ?2, ?3, ?4)",
                                rusqlite::params![link_id, block.url, target_url, link_text]
                            );
                        }
                    }
                    Some(format!("Indexed {} documentation blocks from folder '{}' for framework '{}'.", blocks.len(), source, fw))
                } else {
                    Some("Failed to open database.".to_string())
                }
            }
        }
        "search_framework_specifications" => {
            let query = params.get("query").and_then(|s| s.as_str()).unwrap_or("");
            let category = params.get("category").and_then(|s| s.as_str()).unwrap_or("all");
            let limit = params.get("limit").and_then(|s| s.as_u64()).unwrap_or(5) as usize;
            let db_path = get_db_path();

            let query_emb = rt.block_on(async { llm::generate_embedding(query).await.unwrap_or_default() });

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let docs = crate::search::query_hybrid_documentation(&conn, query, &query_emb, category, limit).unwrap_or_default();
                if docs.is_empty() {
                    Some(format!("No framework documentation found for query '{}'.", query))
                } else {
                    let mut out = format!("=== Framework Docs Results for '{}' ===\n\n", query);
                    for d in docs {
                        out.push_str(&format!("### {} ({})\n{}\n\n", d.title, d.category, d.content));
                    }
                    Some(out)
                }
            } else {
                Some("Failed to open database.".to_string())
            }
        }
        "generate_documentation" => {
            let code = params.get("code_content").and_then(|s| s.as_str()).unwrap_or("");
            let prompt = format!("Generate comprehensive inline documentation and docstrings for:\n\n```\n{}\n```", code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "get_documentation" => {
            let symbol = params.get("symbol_name").and_then(|s| s.as_str()).unwrap_or("");
            Some(format!("Doclookup for symbol '{}': Inspect source code or call search_codebase.", symbol))
        }
        _ => None,
    }
}
