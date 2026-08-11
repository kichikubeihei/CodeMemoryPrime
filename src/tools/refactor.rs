use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::llm;

async fn get_framework_context(code: &str) -> String {
    let config = crate::llm::get_config_from_db_or_env();
    if !config.use_framework_grounding {
        return String::new();
    }
    
    let emb = match crate::llm::generate_embedding(code).await {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    
    let db_path = crate::get_db_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    
    let chunks = match crate::search::query_hybrid_documentation(&conn, code, &emb, "all", config.framework_grounding_chunks) {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    
    if chunks.is_empty() {
        return String::new();
    }
    
    let mut out = String::from("<framework_specs>\n");
    for chunk in chunks {
        out.push_str(&format!("Citation: [{}]({})\n{}\n\n", chunk.title, chunk.url, chunk.content));
    }
    out.push_str("</framework_specs>\n\n");
    out
}


pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "explain_code",
            "description": "Generates clear multi-level code explanations (high-level, architecture, or line-by-line).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Snippet or full code to explain." },
                    "depth": { "type": "string", "description": "'high_level', 'line_by_line', or 'architecture'." }
                },
                "required": ["code_content"]
            }
        }),
        json!({
            "name": "refactor_code",
            "description": "Applies targeted code transformations (clean, extract_function, add_types, convert_async).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Original code." },
                    "goal": { "type": "string", "description": "Refactoring goal description." }
                },
                "required": ["code_content", "goal"]
            }
        }),
        json!({
            "name": "review_code",
            "description": "Performs comprehensive multi-perspective code review (quality, security, performance).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Code to review." }
                },
                "required": ["code_content"]
            }
        }),
        json!({
            "name": "audit_code_hygiene",
            "description": "Audits code snippets for defensive coding standards, input sanitization, hardcoded credentials, and hygiene rules.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Source code." }
                },
                "required": ["code_content"]
            }
        }),
        json!({
            "name": "optimize_code",
            "description": "Analyzes code for performance bottlenecks and algorithmic improvements.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Source code." }
                },
                "required": ["code_content"]
            }
        }),
        json!({
            "name": "generate_tests",
            "description": "Generates unit test suites for functions or modules.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "code_content": { "type": "string", "description": "Source code." },
                    "framework": { "type": "string", "description": "Test framework (e.g. 'pytest', 'cargo test', 'jest')." }
                },
                "required": ["code_content"]
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    let code = params.get("code_content").and_then(|s| s.as_str()).unwrap_or("");

    match name {
        "explain_code" => {
            let depth = params.get("depth").and_then(|s| s.as_str()).unwrap_or("high_level");
            let prompt = format!("Explain this code with depth '{}':\n\n```\n{}\n```", depth, code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "refactor_code" => {
            let goal = params.get("goal").and_then(|s| s.as_str()).unwrap_or("clean");
            let context = rt.block_on(async { get_framework_context(code).await });
            let instruction = if context.is_empty() { "" } else { "If you utilize any information from the provided <framework_specs>, you MUST include inline code comments citing the Title and URL for auditing purposes." };
            let prompt = format!("{}Refactor this code to achieve goal: '{}'. Return modified code and explanation:\n\n```\n{}\n```\n{}", context, goal, code, instruction);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "review_code" => {
            let prompt = format!("Perform a detailed code review covering readability, performance, input handling, and edge cases:\n\n```\n{}\n```", code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "audit_code_hygiene" | "check_security" => {
            let prompt = format!("Audit this code snippet for defensive coding hygiene, hardcoded credentials, input sanitization, and reliability standards:\n\n```\n{}\n```", code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "optimize_code" => {
            let prompt = format!("Analyze and optimize this code for memory efficiency, execution speed, and complexity:\n\n```\n{}\n```", code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "generate_tests" => {
            let fw = params.get("framework").and_then(|s| s.as_str()).unwrap_or("standard unit test");
            let context = rt.block_on(async { get_framework_context(code).await });
            let instruction = if context.is_empty() { "" } else { "If you utilize any information from the provided <framework_specs>, you MUST include inline code comments citing the Title and URL for auditing purposes." };
            let prompt = format!("{}Generate comprehensive unit tests using framework '{}' for:\n\n```\n{}\n```\n{}", context, fw, code, instruction);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_schemas() {
        let schemas = list_schemas();
        assert!(!schemas.is_empty());
        assert!(schemas.iter().any(|s| s["name"] == "refactor_code"));
        assert!(schemas.iter().any(|s| s["name"] == "generate_tests"));
    }
}
