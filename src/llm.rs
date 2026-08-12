use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: String,           // "ollama" or "openai"
    pub pipeline_mode: String,      // "auto", "full_remote", "hybrid_remote_orch", "full_local"
    pub base_url: String,           // Local base URL (e.g. "http://127.0.0.1:11434")
    pub remote_base_url: String,    // Remote Tailscale URL (e.g. "http://100.102.233.128:11434")
    pub gen_model: String,          // Local Coder model (e.g. "qwen2.5-coder:14b")
    pub remote_orch_model: String, // Remote Orchestrator model (e.g. "muse-glimmer:30b")
    pub remote_gen_model: String,  // Remote Coder model (e.g. "qwen2.5-coder:14b")
    pub embed_model: String,        // Embedding model (e.g. "nomic-embed-text")
    pub api_key: String,            // optional API key
    pub use_framework_grounding: bool,
    pub framework_grounding_chunks: usize,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            pipeline_mode: "auto".to_string(),
            base_url: "http://127.0.0.1:11434".to_string(),
            remote_base_url: "".to_string(),
            gen_model: "qwen2.5-coder:14b".to_string(),
            remote_orch_model: "muse-glimmer:30b".to_string(),
            remote_gen_model: "qwen2.5-coder:14b".to_string(),
            embed_model: "nomic-embed-text".to_string(),
            api_key: "".to_string(),
            use_framework_grounding: true,
            framework_grounding_chunks: 3,
        }
    }
}

pub fn get_config_from_db_or_env() -> LlmConfig {
    let db_path = crate::get_db_path();
    let _ = crate::db::init_database(&db_path);
    let mut config = LlmConfig::default();

    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let get_setting = |key: &str| -> Option<String> {
            conn.query_row(
                "SELECT value FROM system_settings WHERE key = ?1",
                rusqlite::params![key],
                |row| row.get(0),
            )
            .ok()
        };

        if let Some(p) = get_setting("llm_provider") { config.provider = p; }
        if let Some(pm) = get_setting("pipeline_mode") { config.pipeline_mode = pm; }
        if let Some(b) = get_setting("llm_base_url") { config.base_url = b; }
        if let Some(rb) = get_setting("llm_remote_base_url") { config.remote_base_url = rb; }
        if let Some(g) = get_setting("llm_gen_model") { config.gen_model = g; }
        if let Some(ro) = get_setting("llm_remote_orch_model") { config.remote_orch_model = ro; }
        if let Some(rg) = get_setting("llm_remote_gen_model") { config.remote_gen_model = rg; }
        if let Some(e) = get_setting("llm_embed_model") { config.embed_model = e; }
        if let Some(k) = get_setting("llm_api_key") { config.api_key = k; }
        if let Some(v) = get_setting("use_framework_grounding") {
            if let Ok(parsed) = v.parse::<bool>() { config.use_framework_grounding = parsed; }
        }
        if let Some(v) = get_setting("framework_grounding_chunks") {
            if let Ok(parsed) = v.parse::<usize>() { 
                config.framework_grounding_chunks = std::cmp::min(parsed, 10);
            }
        }
    }

    // Environment variable overrides
    if let Ok(p) = std::env::var("MCP_LLM_PROVIDER") { config.provider = p; }
    if let Ok(pm) = std::env::var("MCP_PIPELINE_MODE") { config.pipeline_mode = pm; }
    if let Ok(b) = std::env::var("MCP_LLM_BASE_URL") { config.base_url = b; }
    if let Ok(rb) = std::env::var("MCP_LLM_REMOTE_BASE_URL") { config.remote_base_url = rb; }
    if let Ok(g) = std::env::var("MCP_LLM_GEN_MODEL") { config.gen_model = g; }
    if let Ok(ro) = std::env::var("MCP_LLM_REMOTE_ORCH_MODEL") { config.remote_orch_model = ro; }
    if let Ok(rg) = std::env::var("MCP_LLM_REMOTE_GEN_MODEL") { config.remote_gen_model = rg; }
    if let Ok(e) = std::env::var("MCP_LLM_EMBED_MODEL") { config.embed_model = e; }
    if let Ok(k) = std::env::var("MCP_LLM_API_KEY") { config.api_key = k; }

    config
}

pub fn save_config_to_db(config: &LlmConfig) -> Result<()> {
    let db_path = std::env::var("HOME").unwrap_or("/tmp".to_string()) + "/.coder_memory.db";
    let _ = crate::db::init_database(&db_path);
    let conn = rusqlite::Connection::open(&db_path)?;

    let settings = vec![
        ("llm_provider", config.provider.clone()),
        ("pipeline_mode", config.pipeline_mode.clone()),
        ("llm_base_url", config.base_url.clone()),
        ("llm_remote_base_url", config.remote_base_url.clone()),
        ("llm_gen_model", config.gen_model.clone()),
        ("llm_remote_orch_model", config.remote_orch_model.clone()),
        ("llm_remote_gen_model", config.remote_gen_model.clone()),
        ("llm_embed_model", config.embed_model.clone()),
        ("llm_api_key", config.api_key.clone()),
        ("use_framework_grounding", config.use_framework_grounding.to_string()),
        ("framework_grounding_chunks", config.framework_grounding_chunks.to_string()),
    ];

    for (k, v) in settings {
        conn.execute(
            "INSERT OR REPLACE INTO system_settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![k, v],
        )?;
    }

    Ok(())
}

pub async fn query_llm(prompt: &str) -> Result<String> {
    let config = get_config_from_db_or_env();
    query_llm_with_config(prompt, &config).await
}

// Backward compatibility alias
pub async fn query_ollama(prompt: &str) -> Result<String> {
    query_llm(prompt).await
}

pub async fn query_orchestrator(prompt: &str) -> Result<String> {
    let config = get_config_from_db_or_env();
    query_orchestrator_with_config(prompt, &config).await
}

pub async fn query_coder(prompt: &str) -> Result<String> {
    let config = get_config_from_db_or_env();
    query_coder_with_config(prompt, &config).await
}

pub async fn query_orchestrator_with_config(prompt: &str, config: &LlmConfig) -> Result<String> {
    let mode = config.pipeline_mode.to_lowercase();
    
    if (mode == "auto" || mode == "full_remote" || mode == "hybrid_remote_orch") && !config.remote_base_url.trim().is_empty() {
        let remote_url = config.remote_base_url.trim_end_matches('/');
        let model = if !config.remote_orch_model.is_empty() { &config.remote_orch_model } else { &config.gen_model };

        info!("🌐 [Orchestrator Remote-First] Querying Remote Orchestrator ({}) with model '{}'...", remote_url, model);

        let remote_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()?;

        match query_single_endpoint(&remote_client, remote_url, model, &config.provider, &config.api_key, prompt).await {
            Ok(res) => {
                info!("✅ [Orchestrator Success] Received response from Remote Orchestrator ({})", remote_url);
                return Ok(res);
            }
            Err(err) => {
                info!("⚠️ [Orchestrator Failover] Remote Orchestrator ({}) unreachable ({}). Failing over to Local Coder ({})...", remote_url, err, config.base_url);
            }
        }
    }

    // Local Host Execution
    let local_client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let local_url = config.base_url.trim_end_matches('/');
    query_single_endpoint(&local_client, local_url, &config.gen_model, &config.provider, &config.api_key, prompt).await
}

pub async fn query_coder_with_config(prompt: &str, config: &LlmConfig) -> Result<String> {
    let mode = config.pipeline_mode.to_lowercase();

    if (mode == "auto" || mode == "full_remote") && !config.remote_base_url.trim().is_empty() {
        let remote_url = config.remote_base_url.trim_end_matches('/');
        let model = if !config.remote_gen_model.is_empty() { &config.remote_gen_model } else { &config.gen_model };

        info!("⚡ [Coder Remote-First] Querying Remote Coder ({}) with model '{}'...", remote_url, model);

        let remote_client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .connect_timeout(std::time::Duration::from_secs(3))
            .build()?;

        match query_single_endpoint(&remote_client, remote_url, model, &config.provider, &config.api_key, prompt).await {
            Ok(res) => {
                info!("✅ [Coder Remote Success] Received response from Remote Coder ({})", remote_url);
                return Ok(res);
            }
            Err(err) => {
                info!("⚠️ [Coder Failover] Remote Coder ({}) unreachable ({}). Failing over to Local Coder ({})...", remote_url, err, config.base_url);
            }
        }
    }

    // Local Host Execution (Local Coder model)
    let local_client = Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let local_url = config.base_url.trim_end_matches('/');
    query_single_endpoint(&local_client, local_url, &config.gen_model, &config.provider, &config.api_key, prompt).await
}

pub async fn query_llm_with_config(prompt: &str, config: &LlmConfig) -> Result<String> {
    query_coder_with_config(prompt, config).await
}

async fn query_single_endpoint(client: &Client, base_url: &str, model: &str, provider: &str, api_key: &str, prompt: &str) -> Result<String> {
    if provider.to_lowercase() == "openai" {
        // OpenAI-compatible endpoint (/v1/chat/completions or /chat/completions)
        let endpoint = if base_url.ends_with("/v1") {
            format!("{}/chat/completions", base_url)
        } else {
            format!("{}/v1/chat/completions", base_url)
        };

        let request_body = json!({
            "model": model,
            "messages": [
                { "role": "user", "content": prompt }
            ]
        });

        info!("Querying OpenAI-compatible API at {} with model {}", endpoint, model);

        let mut req = client.post(&endpoint).json(&request_body);
        if !api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("OpenAI-compatible API returned status {}: {}", response.status(), response.text().await.unwrap_or_default()));
        }

        let json_resp: Value = response.json().await?;
        if let Some(content) = json_resp.get("choices")
            .and_then(|c| c.get(0))
            .and_then(|m| m.get("message"))
            .and_then(|txt| txt.get("content"))
            .and_then(|s| s.as_str())
        {
            Ok(content.to_string())
        } else {
            Err(anyhow!("Invalid response format from OpenAI-compatible API"))
        }
    } else {
        // Default Ollama native API (/api/generate)
        let endpoint = format!("{}/api/generate", base_url);
        let request_body = json!({
            "model": model,
            "prompt": prompt,
            "stream": false
        });

        info!("Querying Ollama at {} with model {}", endpoint, model);

        let response = client
            .post(&endpoint)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama API returned error: {}", response.status()));
        }

        let json_resp: Value = response.json().await?;
        if let Some(resp) = json_resp.get("response").and_then(|r| r.as_str()) {
            Ok(resp.to_string())
        } else {
            Err(anyhow!("Invalid response format from Ollama"))
        }
    }
}

pub async fn generate_embedding(text: &str) -> Result<Vec<f32>> {
    let config = get_config_from_db_or_env();
    generate_embedding_with_config(text, &config).await
}

pub async fn generate_embedding_with_config(text: &str, config: &LlmConfig) -> Result<Vec<f32>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let base_url = config.base_url.trim_end_matches('/');

    if config.provider.to_lowercase() == "openai" {
        // OpenAI-compatible embeddings endpoint (/v1/embeddings or /embeddings)
        let endpoint = if base_url.ends_with("/v1") {
            format!("{}/embeddings", base_url)
        } else {
            format!("{}/v1/embeddings", base_url)
        };

        let request_body = json!({
            "model": config.embed_model,
            "input": text
        });

        info!("Generating embedding via OpenAI-compatible API at {}", endpoint);

        let mut req = client.post(&endpoint).json(&request_body);
        if !config.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", config.api_key));
        }

        let response = req.send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("OpenAI-compatible embedding API returned status {}", response.status()));
        }

        let json_resp: Value = response.json().await?;
        if let Some(embedding) = json_resp.get("data")
            .and_then(|d| d.get(0))
            .and_then(|item| item.get("embedding"))
            .and_then(|e| e.as_array())
        {
            let vec: Result<Vec<f32>, _> = embedding.iter()
                .map(|v| v.as_f64().ok_or(anyhow!("Invalid float in embedding")))
                .map(|res: Result<f64, anyhow::Error>| res.map(|f| f as f32))
                .collect();
            vec
        } else {
            Err(anyhow!("No embedding field in OpenAI response"))
        }
    } else {
        // Ollama native embeddings API (/api/embeddings)
        let endpoint = format!("{}/api/embeddings", base_url);
        let request_body = json!({
            "model": config.embed_model,
            "prompt": text
        });

        info!("Generating embedding via Ollama at {}", endpoint);

        let response = client
            .post(&endpoint)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama API returned error: {}", response.status()));
        }

        let json_resp: Value = response.json().await?;
        if let Some(embedding) = json_resp.get("embedding").and_then(|e| e.as_array()) {
            let vec: Result<Vec<f32>, _> = embedding.iter()
                .map(|v| v.as_f64().ok_or(anyhow!("Invalid float in embedding")))
                .map(|res: Result<f64, anyhow::Error>| res.map(|f| f as f32))
                .collect();
            vec
        } else {
            Err(anyhow!("No embedding field in Ollama response"))
        }
    }
}

pub async fn check_ollama_connection() -> Result<()> {
    let config = get_config_from_db_or_env();
    let client = Client::new();
    let res = client.get(&format!("{}/api/tags", config.base_url)).send().await?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!("Status code {}", res.status()))
    }
}

pub async fn fetch_available_models(base_url: &str) -> Result<Vec<String>> {
    let client = Client::builder().timeout(std::time::Duration::from_secs(5)).build()?;
    let clean_url = base_url.trim_end_matches('/');

    // 1. Try Ollama tags API (/api/tags)
    let tags_url = format!("{}/api/tags", clean_url);
    if let Ok(res) = client.get(&tags_url).send().await {
        if res.status().is_success() {
            if let Ok(json_resp) = res.json::<Value>().await {
                if let Some(models) = json_resp.get("models").and_then(|m| m.as_array()) {
                    let names: Vec<String> = models.iter()
                        .filter_map(|m| m.get("name").and_then(|s| s.as_str()).map(|s| s.to_string()))
                        .collect();
                    return Ok(names);
                }
            }
        }
    }

    // 2. Try OpenAI models endpoint (/v1/models or /models)
    let models_url = if clean_url.ends_with("/v1") {
        format!("{}/models", clean_url)
    } else {
        format!("{}/v1/models", clean_url)
    };

    if let Ok(res) = client.get(&models_url).send().await {
        if res.status().is_success() {
            if let Ok(json_resp) = res.json::<Value>().await {
                if let Some(data) = json_resp.get("data").and_then(|d| d.as_array()) {
                    let names: Vec<String> = data.iter()
                        .filter_map(|m| m.get("id").and_then(|s| s.as_str()).map(|s| s.to_string()))
                        .collect();
                    return Ok(names);
                }
            }
        }
    }

    Err(anyhow!("Could not reach LLM endpoint at '{}'", base_url))
}

pub async fn auto_detect_llm_setup() -> String {
    let cfg = get_config_from_db_or_env();

    match fetch_available_models(&cfg.base_url).await {
        Ok(models) if !models.is_empty() => {
            let mut out = format!(
                "=== LLM Setup Detected at `{}` ===\n\n", cfg.base_url
            );
            out.push_str(&format!("- **Provider**: {}\n", cfg.provider));
            out.push_str(&format!("- **Active Generation Model**: `{}`\n", cfg.gen_model));
            out.push_str(&format!("- **Active Embedding Model**: `{}`\n\n", cfg.embed_model));

            out.push_str(&format!("### Available Local Models ({} detected):\n", models.len()));
            for m in &models {
                let is_gen = m.contains(&cfg.gen_model);
                let is_emb = m.contains(&cfg.embed_model);
                let tag = match (is_gen, is_emb) {
                    (true, true) => " [ACTIVE GEN & EMBED]",
                    (true, false) => " [ACTIVE GEN]",
                    (false, true) => " [ACTIVE EMBED]",
                    _ => "",
                };
                out.push_str(&format!("  - `{}`{}\n", m, tag));
            }

            out.push_str("\n### To change selected models:\n");
            out.push_str("Call tool `configure_settings` with:\n");
            out.push_str("```json\n{\n  \"action\": \"set\",\n");
            out.push_str(&format!("  \"gen_model\": \"<chosen_gen_model>\",\n  \"embed_model\": \"<chosen_embed_model>\"\n}}\n```"));
            out
        }
        Ok(_) => {
            format!(
                "=== Local LLM Server Detected at `{}` but NO models are installed ===\n\nTo install standard models via Ollama:\n```bash\nollama pull qwen2.5-coder:7b\nollama pull nomic-embed-text\n```\nAfter pulling, re-run `configure_settings`.",
                cfg.base_url
            )
        }
        Err(_) => {
            format!(
                "=== No Active LLM Endpoint Found at `{}` ===\n\nCodeMemoryPrime requires a local LLM or API endpoint to generate code embeddings and answer RAG queries.\n\n### Option A: Use Local Ollama (Recommended / Free)\n1. Install & start Ollama: https://ollama.com\n2. Run in terminal:\n   ```bash\n   ollama pull qwen2.5-coder:7b\n   ollama pull nomic-embed-text\n   ```\n\n### Option B: Use OpenAI / LM Studio / LocalAI\nCall tool `configure_settings`:\n```json\n{{\n  \"action\": \"set\",\n  \"provider\": \"openai\",\n  \"base_url\": \"http://localhost:1234/v1\",\n  \"gen_model\": \"your-model-name\",\n  \"embed_model\": \"your-embed-name\",\n  \"api_key\": \"lm-studio\"\n}}\n```",
                cfg.base_url
            )
        }
    }
}
