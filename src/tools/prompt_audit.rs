use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::get_db_path;
use rusqlite::{Connection, params};
use std::fs;

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "audit_prompt_quality",
            "description": "Performs static prompt quality analysis, ambiguity checking, hallucination risk assessment, and best-practice optimization suggestions. Auto-discovers LLM prompts across project if omitted.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt_content": { "type": "string", "description": "System prompt text to audit (optional if file_path or project_name is provided)." },
                    "file_path": { "type": "string", "description": "Path to prompt file (optional)." },
                    "project_name": { "type": "string", "description": "Project identifier to auto-discover all indexed prompts (optional)." }
                }
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, _rt: &Runtime) -> Option<String> {
    match name {
        "audit_prompt_quality" => Some(handle_audit_prompt_quality(params)),
        _ => None,
    }
}

pub fn audit_prompt_string(prompt: &str, title: &str) -> String {
    let mut score = 100i32;
    let mut findings: Vec<String> = Vec::new();
    let lower = prompt.to_lowercase();

    // 1. Grounding & Anti-Hallucination Check
    let has_grounding = lower.contains("strictly") 
        || lower.contains("only use") 
        || lower.contains("based on")
        || lower.contains("provided context")
        || lower.contains("do not invent")
        || lower.contains("don't invent")
        || lower.contains("if unknown")
        || lower.contains("i don't know")
        || lower.contains("insufficient information");

    if !has_grounding {
        score -= 25;
        findings.push("⚠️ **Missing Anti-Hallucination Guardrail**: Prompt does not explicitly restrict answers to provided context or mandate declaring 'I do not know' when context is missing.".to_string());
    }

    // 2. Ambiguity & Vagueness Check
    let vague_terms = vec!["do a good job", "be accurate", "be smart", "answer well", "try your best", "help the user"];
    for term in vague_terms {
        if lower.contains(term) {
            score -= 10;
            findings.push(format!("⚠️ **Vague Phrasing Detected**: Found generic phrase '{}'. Replace with concrete output specifications.", term));
        }
    }

    // 3. Structured Format Constraint
    let has_format = lower.contains("json") 
        || lower.contains("format") 
        || lower.contains("markdown") 
        || lower.contains("schema")
        || lower.contains("xml")
        || lower.contains("headers");

    if !has_format {
        score -= 15;
        findings.push("⚠️ **Unconstrained Output Format**: Prompt does not mandate a specific structured output format (JSON, Markdown schema, XML tags).".to_string());
    }

    // 4. Role & Persona Definition
    let has_role = lower.contains("you are") || lower.contains("role:") || lower.contains("act as");
    if !has_role {
        score -= 10;
        findings.push("ℹ️ **Missing Role Context**: Prompt lacks a clear identity declaration (e.g. 'You are an expert compiler engineer...').".to_string());
    }

    // 5. Token Efficiency
    let token_count = prompt.split_whitespace().count();
    let fluff_terms = vec!["please", "kindly", "as an ai model", "thank you", "feel free to"];
    for fluff in fluff_terms {
        if lower.contains(fluff) {
            score -= 5;
            findings.push(format!("ℹ️ **Token Waste**: Found conversational filler word/phrase '{}'. Remove to save token costs.", fluff));
        }
    }

    let clamped_score = score.max(0).min(100);

    let mut out = format!("### Prompt Audit: `{}`\n", title);
    out.push_str(&format!("- **Prompt Quality Score**: **{} / 100**\n", clamped_score));
    out.push_str(&format!("- **Estimated Word Count**: {} words\n\n", token_count));

    if findings.is_empty() {
        out.push_str("✅ **Prompt meets AI Engineering Best Practices!** No high-priority ambiguity or hallucination risks detected.\n\n");
    } else {
        out.push_str("#### Findings & Optimization Checklist:\n");
        for f in &findings {
            out.push_str(&format!("- {}\n", f));
        }
        out.push_str("\n");
    }

    out
}

fn handle_audit_prompt_quality(params: &Value) -> String {
    let raw_prompt = params.get("prompt_content").and_then(|s| s.as_str());
    let file_path = params.get("file_path").and_then(|s| s.as_str());
    let project_name = params.get("project_name").and_then(|s| s.as_str());

    if let Some(prompt) = raw_prompt {
        return format!("=== Static AI Prompt Quality Audit ===\n\n{}", audit_prompt_string(prompt, "User Provided Prompt"));
    }

    if let Some(path) = file_path {
        match fs::read_to_string(path) {
            Ok(content) => return format!("=== Static AI Prompt Quality Audit ===\n\n{}", audit_prompt_string(&content, path)),
            Err(e) => return format!("File Read Error: Unable to open '{}': {}", path, e),
        }
    }

    // Auto-discover LLM prompts across indexed codebase
    let db_path = get_db_path();
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return format!("Database Error: Unable to open database at '{}': {}", db_path, e),
    };

    let map_row = |row: &rusqlite::Row| -> rusqlite::Result<(String, String, String)> {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    };

    let chunks: Vec<(String, String, String)> = if let Some(proj) = project_name {
        let mut stmt = match conn.prepare("SELECT file_path, name, code_content FROM code_chunks WHERE project_name = ?1 AND (chunk_type = 'llm_prompt' OR chunk_type = 'llm_call') LIMIT 20") {
            Ok(s) => s,
            Err(e) => return format!("Query Error: {}", e),
        };
        stmt.query_map(params![proj], map_row).map(|r| r.flatten().collect()).unwrap_or_default()
    } else {
        let mut stmt = match conn.prepare("SELECT file_path, name, code_content FROM code_chunks WHERE chunk_type = 'llm_prompt' OR chunk_type = 'llm_call' LIMIT 20") {
            Ok(s) => s,
            Err(e) => return format!("Query Error: {}", e),
        };
        stmt.query_map([], map_row).map(|r| r.flatten().collect()).unwrap_or_default()
    };

    if chunks.is_empty() {
        return "=== Auto-Discovered Prompt Quality Audit ===\n\nNo indexed LLM prompts or API calls (`chunk_type = 'llm_prompt'`) were auto-discovered in the project database. Pass `prompt_content` directly or re-index your codebase.".to_string();
    }

    let mut output = format!("=== Auto-Discovered Prompt Quality Audit ({} Prompts Found) ===\n\n", chunks.len());
    for (path, name, content) in &chunks {
        let title = format!("{}: {}", path, name);
        output.push_str(&audit_prompt_string(content, &title));
        output.push_str("---\n\n");
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_prompt_string_quality() {
        let weak_prompt = "Please do a good job and be smart when answering questions. Help the user as an AI model.";
        let report = audit_prompt_string(weak_prompt, "Weak Test Prompt");
        assert!(report.contains("Missing Anti-Hallucination Guardrail"));
        assert!(report.contains("Vague Phrasing Detected"));
        assert!(report.contains("Token Waste"));

        let strong_prompt = "You are an expert compiler engineer. Role: Auditor. Answer strictly based on the provided context. If context is insufficient, reply 'I do not have enough information'. Output format must be valid JSON.";
        let strong_report = audit_prompt_string(strong_prompt, "Strong Test Prompt");
        assert!(strong_report.contains("Prompt Quality Score**: **100 / 100**"));
    }
}
