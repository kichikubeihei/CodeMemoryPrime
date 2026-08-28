use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub modified_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RosterSyncResult {
    pub endpoint: String,
    pub models_found: Vec<OllamaModel>,
    pub gemini_md_path: String,
    pub updated: bool,
    pub message: String,
}

/// Queries the Ollama API at the given endpoint, fetches the live model list,
/// and rewrites the Tailscale model table in ~/.gemini/config/GEMINI.md.
pub fn sync_tailscale_model_roster(
    endpoint: Option<&str>,
    gemini_md_path: Option<&str>,
) -> std::result::Result<RosterSyncResult, String> {
    let host = endpoint.unwrap_or("http://100.102.233.128:11434");
    let md_path = gemini_md_path.unwrap_or_else(|| {
        // default to ~/.gemini/config/GEMINI.md
        Box::leak(
            format!(
                "{}/.gemini/config/GEMINI.md",
                std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
            )
            .into_boxed_str(),
        )
    });

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    let tags_url = format!("{}/api/tags", host);
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let models: Vec<OllamaModel> = rt.block_on(async {
        let resp = client
            .get(&tags_url)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let body: Value = resp.json().await.map_err(|e| e.to_string())?;
        let model_arr = body
            .get("models")
            .and_then(|m| m.as_array())
            .cloned()
            .unwrap_or_default();
        Ok::<Vec<OllamaModel>, String>(
            model_arr
                .into_iter()
                .filter_map(|m| {
                    Some(OllamaModel {
                        name: m.get("name")?.as_str()?.to_string(),
                        size: m.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
                        modified_at: m
                            .get("modified_at")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string()),
                    })
                })
                .collect(),
        )
    })?;

    if models.is_empty() {
        return Ok(RosterSyncResult {
            endpoint: host.to_string(),
            models_found: vec![],
            gemini_md_path: md_path.to_string(),
            updated: false,
            message: "No models returned from Ollama endpoint. Is the workstation reachable over Tailscale?".to_string(),
        });
    }

    // Build best_for hints based on known model families
    let best_for_hint = |name: &str| -> &str {
        let n = name.to_lowercase();
        if n.contains("coder") && n.contains("32b") { "Rust, Svelte, large file refactoring" }
        else if n.contains("coder") && n.contains("14b") { "Fast code search, snippet generation" }
        else if n.contains("coder") { "Code generation and refactoring" }
        else if n.contains("qwen3") || (n.contains("qwen") && n.contains("30b")) { "Long document analysis, reasoning chains" }
        else if n.contains("deepseek") && n.contains("r1") { "Algorithmic debugging, math, step-by-step" }
        else if n.contains("llama3.3") || n.contains("llama3") && n.contains("70b") { "High-complexity architecture, system design" }
        else if n.contains("vision") { "UI screenshots, image-to-code" }
        else if n.contains("gemma4") || n.contains("gemma") && n.contains("27b") { "Multimodal UI review, vision + code" }
        else if n.contains("muse") || n.contains("glimmer") { "Creative writing, Altalune novelist content" }
        else if n.contains("llama") { "General reasoning and instruction following" }
        else { "General purpose inference" }
    };

    let context_hint = |name: &str| -> &str {
        let n = name.to_lowercase();
        if n.contains("qwen3") || n.contains("gemma4") { "262K" }
        else if n.contains("llama3.3") || n.contains("70b") || n.contains("muse") { "131K" }
        else if n.contains("deepseek") { "65K" }
        else if n.contains("vision") { "16K" }
        else { "32K" }
    };

    // Build the new table markdown
    let mut table = String::from(
        "| Model | Best For | Context |\n|---|---|---|\n"
    );
    for m in &models {
        table.push_str(&format!(
            "| `{}` | {} | {} |\n",
            m.name,
            best_for_hint(&m.name),
            context_hint(&m.name)
        ));
    }

    // Read existing GEMINI.md
    let existing = std::fs::read_to_string(md_path)
        .unwrap_or_else(|_| String::new());

    // Replace the model table block between the sentinel comments
    let start_marker = "<!-- TAILSCALE_ROSTER_START -->";
    let end_marker = "<!-- TAILSCALE_ROSTER_END -->";

    let new_content = if existing.contains(start_marker) && existing.contains(end_marker) {
        // Splice in the new table
        let before = &existing[..existing.find(start_marker).unwrap() + start_marker.len()];
        let after_start = existing.find(end_marker).unwrap();
        let after = &existing[after_start..];
        format!("{}\n{}\n{}", before, table, after)
    } else {
        // Sentinel markers not present — append a new section at the end
        format!(
            "{}\n\n## Auto-Synced Tailscale Model Roster\n\n{}\n{}\n{}\n",
            existing,
            start_marker,
            table,
            end_marker
        )
    };

    // Write back
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(md_path)
        .map_err(|e| format!("Failed to write GEMINI.md: {}", e))?;
    f.write_all(new_content.as_bytes())
        .map_err(|e| format!("Failed to write GEMINI.md bytes: {}", e))?;

    Ok(RosterSyncResult {
        endpoint: host.to_string(),
        models_found: models.clone(),
        gemini_md_path: md_path.to_string(),
        updated: true,
        message: format!(
            "GEMINI.md updated with {} live models from {}. Restart Antigravity to pick up changes.",
            models.len(),
            host
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roster_sync_result_fields() {
        let result = RosterSyncResult {
            endpoint: "http://100.102.233.128:11434".to_string(),
            models_found: vec![OllamaModel {
                name: "qwen2.5-coder:32b".to_string(),
                size: 19_500_000_000,
                modified_at: None,
            }],
            gemini_md_path: "/tmp/GEMINI.md".to_string(),
            updated: true,
            message: "OK".to_string(),
        };
        assert!(result.updated);
        assert_eq!(result.models_found.len(), 1);
    }
}
