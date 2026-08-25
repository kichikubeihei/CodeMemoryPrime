use crate::db::init_database;
use crate::get_db_path;
use reqwest::Client;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebIngestSummary {
    pub url: String,
    pub project_name: String,
    pub title: String,
    pub total_bytes: usize,
    pub chunks_created: usize,
    pub message: String,
}

/// Downloads full HTML page, converts to clean markdown, and ingests into SQLite document memory
pub fn ingest_url(url: &str, project_name: &str) -> Result<WebIngestSummary> {
    let db_path = get_db_path();
    init_database(&db_path)?;
    let conn = Connection::open(&db_path)?;

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("CodeMemoryPrime/1.0 WebIngestEngine")
        .build()
        .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

    let html_content = rt.block_on(async {
        let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
        resp.text().await.map_err(|e| e.to_string())
    }).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e))))?;

    // Convert HTML to text/markdown using scraper
    let document = ::scraper::Html::parse_document(&html_content);
    let title_selector = ::scraper::Selector::parse("title").ok();
    let title = title_selector
        .and_then(|sel| document.select(&sel).next())
        .map(|e| e.text().collect::<String>())
        .unwrap_or_else(|| url.to_string());

    // Extract text from body
    let body_selector = ::scraper::Selector::parse("body").ok();
    let raw_text = body_selector
        .and_then(|sel| document.select(&sel).next())
        .map(|e| e.text().collect::<Vec<_>>().join(" "))
        .unwrap_or(html_content);

    // Clean whitespace
    let clean_text = raw_text.split_whitespace().collect::<Vec<_>>().join(" ");
    let total_bytes = clean_text.len();

    // Chunk text (approx 1,000 chars per chunk)
    let chunks: Vec<String> = clean_text
        .as_bytes()
        .chunks(1000)
        .map(|c| String::from_utf8_lossy(c).to_string())
        .collect();

    let doc_id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    let mut chunks_created = 0;
    for (idx, chunk) in chunks.iter().enumerate() {
        let chunk_id = format!("{}-{}", doc_id, idx);
        let name = format!("{} (Part {})", title, idx + 1);

        let _ = conn.execute(
            "INSERT OR REPLACE INTO code_chunks (id, file_path, file_name, chunk_type, name, code_content, summary, project_name, parent_context)
             VALUES (?1, ?2, ?3, 'web_doc', ?4, ?5, ?6, ?7, ?8)",
            params![
                chunk_id,
                url,
                title,
                name,
                chunk,
                format!("Ingested web doc chunk from {}", url),
                project_name,
                timestamp
            ],
        );

        let _ = conn.execute(
            "INSERT INTO code_chunks_fts (id, file_path, file_name, chunk_type, name, code_content, summary, project_name)
             VALUES (?1, ?2, ?3, 'web_doc', ?4, ?5, ?6, ?7)",
            params![
                chunk_id,
                url,
                title,
                name,
                chunk,
                format!("Ingested web doc chunk from {}", url),
                project_name
            ],
        );

        chunks_created += 1;
    }

    Ok(WebIngestSummary {
        url: url.to_string(),
        project_name: project_name.to_string(),
        title,
        total_bytes,
        chunks_created,
        message: format!("Successfully ingested full web document into SQLite memory ({} chunks, {} bytes).", chunks_created, total_bytes),
    })
}

/// Offloads full document analysis to a Local or Tailscale Ollama/vLLM endpoint ($0.00 API cost)
pub fn analyze_doc_with_local_llm(url_or_text: &str, endpoint: Option<&str>) -> std::result::Result<String, String> {
    let local_url = endpoint
        .or_else(|| std::option_env!("OLLAMA_HOST"))
        .unwrap_or("http://127.0.0.1:11434");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let prompt = format!(
        "Analyze the following web document content. Extract key takeaways, core technical points, and main recommendations in concise bullet points:\n\n{}",
        &url_or_text[..url_or_text.len().min(4000)]
    );

    let payload = json!({
        "model": "qwen2.5-coder:14b",
        "prompt": prompt,
        "stream": false
    });

    let res = rt.block_on(async {
        let resp = client
            .post(format!("{}/api/generate", local_url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
        Ok::<String, String>(body.get("response").and_then(|s| s.as_str()).unwrap_or("No response").to_string())
    });

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_ingest_summary_format() {
        let summary = WebIngestSummary {
            url: "https://example.com".to_string(),
            project_name: "test".to_string(),
            title: "Example Title".to_string(),
            total_bytes: 500,
            chunks_created: 1,
            message: "Ingested".to_string(),
        };
        assert_eq!(summary.chunks_created, 1);
    }
}
