use serde_json::{json, Value};
use tokio::runtime::Runtime;
use crate::llm;

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
            "name": "check_security",
            "description": "Scans code snippets for OWASP vulnerabilities and security flaws.",
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
            let prompt = format!("Refactor this code to achieve goal: '{}'. Return modified code and explanation:\n\n```\n{}\n```", goal, code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "review_code" => {
            let prompt = format!("Perform a detailed code review covering readability, performance, security, and edge cases:\n\n```\n{}\n```", code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        "check_security" => {
            let prompt = format!("Scan this code for security vulnerabilities, injection risks, unsafe memory access, or credentials:\n\n```\n{}\n```", code);
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
            let prompt = format!("Generate comprehensive unit tests using framework '{}' for:\n\n```\n{}\n```", fw, code);
            let resp = rt.block_on(async { llm::query_ollama(&prompt).await.unwrap_or_default() });
            Some(resp)
        }
        _ => None,
    }
}
