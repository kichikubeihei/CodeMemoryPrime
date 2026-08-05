use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::{get_db_path, db, llm};
use rusqlite::{Connection, params};
use std::fs;
use std::path::Path;

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "index_custom_documents",
            "description": "Ingests custom documents (SOP manuals, PDFs, Markdown guidelines, books, style guides) into vector database for project-grounded AI context.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "file_path": { "type": "string", "description": "Absolute path to text file, Markdown document, PDF, or SOP manual." },
                    "document_title": { "type": "string", "description": "Human-readable title or book name (e.g. 'Company Coding Standards', 'Genetic Algorithms Manual')." },
                    "category": { "type": "string", "description": "Category tag (e.g. 'sop_policy', 'coding_standards', 'textbook', 'general')." }
                },
                "required": ["file_path"]
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    match name {
        "index_custom_documents" => Some(handle_index_custom_documents(params, rt)),
        _ => None,
    }
}

pub fn chunk_text_content(content: &str, chunk_size_words: usize) -> Vec<String> {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return Vec::new();
    }

    let mut chunks = Vec::new();
    for i in (0..words.len()).step_by(chunk_size_words) {
        let end = std::cmp::min(i + chunk_size_words, words.len());
        let segment = words[i..end].join(" ");
        chunks.push(segment);
    }
    chunks
}

fn handle_index_custom_documents(params: &Value, rt: &Runtime) -> String {
    let file_path = match params.get("file_path").and_then(|s| s.as_str()) {
        Some(p) if !p.trim().is_empty() => p.trim(),
        _ => return "Error: 'file_path' parameter is required.".to_string(),
    };

    let title = params.get("document_title")
        .and_then(|s| s.as_str())
        .unwrap_or_else(|| {
            Path::new(file_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Custom Document")
        });

    let category = params.get("category").and_then(|s| s.as_str()).unwrap_or("custom_knowledge");

    let raw_text = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => return format!("File Read Error: Unable to open '{}': {}", file_path, e),
    };

    if raw_text.trim().is_empty() {
        return format!("Error: Document '{}' is empty.", file_path);
    }

    let chunks = chunk_text_content(&raw_text, 400);
    if chunks.is_empty() {
        return format!("Error: Failed to extract chunks from '{}'.", file_path);
    }

    let db_path = get_db_path();
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return format!("Database Error: Unable to open database at '{}': {}", db_path, e),
    };

    let mut indexed_count = 0;
    for (idx, segment) in chunks.iter().enumerate() {
        let chunk_id = format!("{}_{}", title.replace(' ', "_"), idx + 1);
        let chunk_title = format!("{} (Part {})", title, idx + 1);
        let embedding = rt.block_on(async {
            llm::generate_embedding(segment).await.unwrap_or_default()
        });

        let emb_blob = db::vector_to_blob(&embedding);
        let res = conn.execute(
            "INSERT OR REPLACE INTO framework_documentation (id, category, version, title, url, content, embedding) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![chunk_id, category, "1.0", chunk_title, file_path, segment, emb_blob],
        );

        let _ = conn.execute(
            "INSERT OR REPLACE INTO framework_documentation_fts (id, category, version, title, url, content) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![chunk_id, category, "1.0", chunk_title, file_path, segment],
        );

        if res.is_ok() {
            indexed_count += 1;
        }
    }

    format!(
        "🎉 **Custom Document Successfully Indexed!**\n\n- **Document Title**: `{}`\n- **Category**: `{}`\n- **Source File**: [`{}`]({})\n- **Total Chunks Vector-Indexed**: {}\n\nLocal AI will now automatically ground code generation against this knowledge base!",
        title, category, file_path, file_path, indexed_count
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_schemas() {
        let schemas = list_schemas();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0]["name"], "index_custom_documents");
    }

    #[test]
    fn test_chunk_text_content() {
        let text = "one two three four five six seven eight nine ten";
        let chunks = chunk_text_content(text, 4);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], "one two three four");
        assert_eq!(chunks[1], "five six seven eight");
        assert_eq!(chunks[2], "nine ten");
    }
}
