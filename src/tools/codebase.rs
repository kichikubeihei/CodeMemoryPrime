use serde_json::{json, Value};
use tokio::runtime::Runtime;
use tracing::info;
use crate::{db, llm, parser, search, license, get_db_path};

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "index_workspace",
            "description": "Recursively scans and indexes a codebase folder into memory. ALWAYS call this first when starting work on a new project, or when search results seem stale or miss relevant code. Parses files into semantic chunks, generates vector embeddings via Ollama, and stores them in the local SQLite memory database. Clears old entries for the project on each call to avoid duplicates.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "directory_path": { "type": "string", "description": "Absolute path to the project root directory to index." },
                    "project_name": { "type": "string", "description": "Unique project identifier. Use a consistent name (e.g. 'altalune', 'mcp-coder-memory') across all tool calls for this project." }
                },
                "required": ["directory_path", "project_name"]
            }
        }),
        json!({
            "name": "search_codebase",
            "description": "Performs RRF (Reciprocal Rank Fusion) hybrid search — combining semantic vector similarity and keyword FTS5 — across indexed code chunks. Use this INSTEAD of reading files directly. Returns ranked source code snippets with file paths, dependency cross-references, and import graphs. If no results are returned, the workspace may not be indexed yet — call index_workspace first.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Natural language or code-like description of what you are looking for." },
                    "project_name": { "type": "string", "description": "Project to search within. Use 'all' to search across all indexed projects." },
                    "limit": { "type": "integer", "description": "Maximum number of code chunks to return (default 10)." },
                    "include_surrounding_lines": { "type": "boolean", "description": "If true, fetches full file snippet context around matches." }
                },
                "required": ["query"]
            }
        }),
        json!({
            "name": "get_dependencies",
            "description": "Retrieves internal and external module dependencies for a project or file. Returns import paths, imported-by caller relationships, and module linkage.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "project_name": { "type": "string", "description": "Project identifier or 'all'." },
                    "output_format": { "type": "string", "description": "'text' or 'json' (default 'text')." }
                }
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    match name {
        "index_workspace" => {
            let dir = params.get("directory_path").and_then(|s| s.as_str()).unwrap_or("");
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("");

            if !std::path::Path::new(dir).exists() {
                Some(format!("Directory '{}' does not exist.", dir))
            } else {
                let db_path = get_db_path();
                let _ = db::init_database(&db_path);

                let mut files_indexed = 0usize;
                let mut chunks_indexed = 0usize;
                let mut errors = Vec::new();

                let extensions = ["py", "rs", "js", "jsx", "ts", "tsx", "svelte", "vue", "css", "html", "toml", "yaml", "yml", "json", "go", "java", "c", "cpp", "h", "hpp"];
                let skip_dirs = [
                    "target", "node_modules", ".git", "__pycache__", ".venv", "venv", "env",
                    "dist", "build", ".next", ".cache", ".idea", ".vscode", ".gemini", ".claude",
                    ".cursor", "brain", "scratch", "logs", "tmp", "temp", "vendor", "coverage"
                ];
                let skip_files = [
                    "Cargo.lock", "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
                    ".DS_Store", "thumbs.db"
                ];

                let mut file_paths: Vec<std::path::PathBuf> = Vec::new();
                if let Ok(_walker) = std::fs::read_dir(dir) {
                    let mut queue: Vec<std::path::PathBuf> = vec![std::path::PathBuf::from(dir)];
                    while let Some(current_dir) = queue.pop() {
                        if let Ok(entries) = std::fs::read_dir(&current_dir) {
                            for entry in entries.flatten() {
                                let path = entry.path();
                                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                                if path.is_dir() {
                                    if !skip_dirs.contains(&fname) && !fname.starts_with('.') {
                                        queue.push(path);
                                    }
                                } else if path.is_file() {
                                    if skip_files.contains(&fname) || fname.ends_with(".log") || fname.ends_with(".db") || fname.ends_with(".sqlite") || fname.ends_with(".env") || fname.ends_with(".min.js") || fname.ends_with(".min.css") {
                                        continue;
                                    }

                                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                                    if extensions.contains(&ext) {
                                        file_paths.push(path);
                                    }
                                }
                            }
                        }
                    }
                }

                info!("Indexing {} files for project '{}'", file_paths.len(), project);

                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    let mut existing_embeddings: std::collections::HashMap<(String, String), (String, Vec<u8>)> = std::collections::HashMap::new();
                    if let Ok(mut stmt) = conn.prepare("SELECT file_path, name, chunk_hash, embedding FROM code_chunks WHERE project_name = ?1") {
                        if let Ok(mut rows) = stmt.query(rusqlite::params![project]) {
                            while let Ok(Some(row)) = rows.next() {
                                let fp: String = row.get(0).unwrap_or_default();
                                let nm: String = row.get(1).unwrap_or_default();
                                let ch: String = row.get(2).unwrap_or_default();
                                let emb: Vec<u8> = row.get(3).unwrap_or_default();
                                existing_embeddings.insert((fp, nm), (ch, emb));
                            }
                        }
                    }

                    let _ = conn.execute("DELETE FROM code_chunks WHERE project_name = ?1", rusqlite::params![project]);
                    let _ = conn.execute("DELETE FROM code_chunks_fts WHERE project_name = ?1", rusqlite::params![project]);
                    let _ = conn.execute("DELETE FROM code_dependencies WHERE project_name = ?1", rusqlite::params![project]);

                    for file_path in &file_paths {
                        let path_str = file_path.to_str().unwrap_or("");
                        match std::fs::read_to_string(file_path) {
                            Ok(content) => {
                                let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
                                let chunks = parser::parse_file_chunks(path_str, &content);
                                let imports = parser::extract_imports(path_str, &content);

                                for import in &imports {
                                    let dep_id = uuid::Uuid::new_v4().to_string();
                                    let _ = conn.execute(
                                        "INSERT OR IGNORE INTO code_dependencies (id, project_name, source_file, import_path) VALUES (?1, ?2, ?3, ?4)",
                                        rusqlite::params![dep_id, project, path_str, import]
                                    );
                                }

                                for chunk in chunks {
                                    let chunk_hash = license::calculate_chunk_hash(&chunk.code_content);
                                    let key = (path_str.to_string(), chunk.name.clone());
                                    
                                    let blob = if let Some((old_hash, old_blob)) = existing_embeddings.get(&key) {
                                        if old_hash == &chunk_hash && !old_blob.is_empty() {
                                            old_blob.clone()
                                        } else {
                                            let embed_text = format!("Signature: {} {}\nSummary: {}\nContext: {}\nCode:\n{}", chunk.chunk_type, chunk.name, chunk.summary, chunk.parent_context, &chunk.code_content[..chunk.code_content.len().min(1000)]);
                                            let embedding = rt.block_on(async {
                                                llm::generate_embedding(&embed_text).await.unwrap_or_else(|_| Vec::new())
                                            });
                                            db::vector_to_blob(&embedding)
                                        }
                                    } else {
                                        let embed_text = format!("Signature: {} {}\nSummary: {}\nContext: {}\nCode:\n{}", chunk.chunk_type, chunk.name, chunk.summary, chunk.parent_context, &chunk.code_content[..chunk.code_content.len().min(1000)]);
                                        let embedding = rt.block_on(async {
                                            llm::generate_embedding(&embed_text).await.unwrap_or_else(|_| Vec::new())
                                        });
                                        db::vector_to_blob(&embedding)
                                    };

                                    let chunk_id = uuid::Uuid::new_v4().to_string();

                                    let _ = conn.execute(
                                        "INSERT OR REPLACE INTO code_chunks (id, file_path, file_name, chunk_type, name, code_content, summary, embedding, project_name, parent_context, chunk_hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                                        rusqlite::params![chunk_id, path_str, file_name, chunk.chunk_type, chunk.name, chunk.code_content, chunk.summary, blob, project, chunk.parent_context, chunk_hash]
                                    );
                                    let _ = conn.execute(
                                        "INSERT INTO code_chunks_fts (id, file_path, file_name, chunk_type, name, code_content, summary, project_name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                                        rusqlite::params![chunk_id, path_str, file_name, chunk.chunk_type, &chunk.name, &chunk.code_content, &chunk.summary, project]
                                    );
                                    chunks_indexed += 1;
                                }
                                files_indexed += 1;
                            }
                            Err(e) => errors.push(format!("{}: {}", path_str, e)),
                        }
                    }
                }

                let checkpoint_msg = crate::tools::shell_git::create_git_checkpoint(&format!("Baseline initial workspace index for '{}'", project), dir);

                info!("Indexed {} files, {} chunks for project '{}'", files_indexed, chunks_indexed, project);
                let mut result = format!("Indexing complete for project '{}':\n- Files indexed: {}\n- Chunks stored: {}\n- {}", project, files_indexed, chunks_indexed, checkpoint_msg);
                if !errors.is_empty() {
                    result.push_str(&format!("\n- Errors ({}):\n", errors.len()));
                    for e in errors.iter().take(5) { result.push_str(&format!("  - {}\n", e)); }
                }
                Some(result)
            }
        }
        "search_codebase" => {
            let query = params.get("query").and_then(|s| s.as_str()).unwrap_or("");
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let limit = params.get("limit").and_then(|s| s.as_u64()).unwrap_or(10) as usize;

            let db_path = get_db_path();

            let embedding = rt.block_on(async {
                llm::generate_embedding(query).await.unwrap_or_else(|_| Vec::new())
            });

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                match search::query_hybrid_codebase(&conn, query, &embedding, project, limit) {
                    Ok(results) => {
                        if results.is_empty() {
                            let chunk_count: i64 = conn.query_row(
                                "SELECT COUNT(*) FROM code_chunks WHERE project_name = ?1 OR ?1 = 'all'",
                                rusqlite::params![project],
                                |r| r.get(0)
                            ).unwrap_or(0);

                            if chunk_count == 0 {
                                let available_projects: Vec<String> = conn.prepare("SELECT DISTINCT project_name FROM code_chunks")
                                    .map(|mut s| s.query_map([], |r| r.get(0)).unwrap().flatten().collect())
                                    .unwrap_or_default();

                                Some(format!(
                                    "[Notice] Project '{}' has no indexed code chunks yet.\n\nTo index this project, call tool `index_workspace`:\n```json\n{{\n  \"directory_path\": \"/path/to/project\",\n  \"project_name\": \"{}\"\n}}\n```\n{}",
                                    project, project,
                                    if available_projects.is_empty() { "No other projects have been indexed yet.".to_string() } else { format!("Currently indexed projects: {}", available_projects.join(", ")) }
                                ))
                            } else {
                                Some(format!("No matches found for query '{}' in project '{}' (searched {} indexed code chunks). Try broadening your search query or re-indexing if files were modified recently.", query, project, chunk_count))
                            }
                        } else {
                            let mut out = format!("Found {} results for '{}':\n\n", results.len(), query);
                            for res in &results {
                                out.push_str(&format!("### {} `{}` in `{}`\n", res.chunk_type, res.name, res.file_name));
                                out.push_str(&format!("**File:** `{}`\n", res.file_path));
                                if !res.parent_context.is_empty() {
                                    out.push_str(&format!("**Context:** `{}`\n", res.parent_context));
                                }
                                if !res.imports.is_empty() {
                                    out.push_str(&format!("**Imports:** {}\n", res.imports.join(", ")));
                                }
                                if !res.imported_by.is_empty() {
                                    out.push_str(&format!("**Imported By:** {}\n", res.imported_by.join(", ")));
                                }
                                out.push_str("```\n");
                                out.push_str(&res.code_content);
                                out.push_str("\n```\n\n");
                            }
                            Some(out)
                        }
                    }
                    Err(e) => Some(format!("Database error during search: {}", e)),
                }
            } else {
                Some("Failed to connect to memory database.".to_string())
            }
        }
        "get_dependencies" => {
            let project = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let fmt = params.get("output_format").and_then(|s| s.as_str()).unwrap_or("text");
            let db_path = get_db_path();

            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let mut sql = "SELECT source_file, import_path FROM code_dependencies".to_string();
                let params_vec: Vec<&dyn rusqlite::ToSql> = if project != "all" {
                    sql.push_str(" WHERE project_name = ?1");
                    vec![&project]
                } else {
                    vec![]
                };

                let mut stmt = conn.prepare(&sql).unwrap();
                let mut rows = stmt.query(rusqlite::params_from_iter(params_vec)).unwrap();
                let mut deps: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

                while let Ok(Some(row)) = rows.next() {
                    let src: String = row.get(0).unwrap_or_default();
                    let imp: String = row.get(1).unwrap_or_default();
                    deps.entry(src).or_default().push(imp);
                }

                if fmt == "json" {
                    Some(json!(deps).to_string())
                } else {
                    let mut out = format!("Dependency graph for project '{}':\n\n", project);
                    for (src, imps) in &deps {
                        out.push_str(&format!("- `{}`:\n", src));
                        for imp in imps {
                            out.push_str(&format!("  -> `{}`\n", imp));
                        }
                    }
                    Some(out)
                }
            } else {
                Some("Failed to connect to database.".to_string())
            }
        }
        _ => None,
    }
}
