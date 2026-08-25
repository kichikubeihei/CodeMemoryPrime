pub mod codebase;
pub mod memory;
pub mod file_ops;
pub mod shell_git;
pub mod refactor;
pub mod docs;
pub mod plugins;
pub mod system;
pub mod prompt_audit;
pub mod custom_docs;

use serde_json::Value;
use tokio::runtime::Runtime;

pub fn list_all_tools() -> Vec<Value> {
    let mode = std::env::var("CMP_MODE").unwrap_or_else(|_| "orchestrator".to_string()).to_lowercase();
    if mode == "orchestrator" || mode == "default" {
        return vec![
            serde_json::json!({
                "name": "orchestrate_code_search",
                "description": "Specialist agent orchestrator for codebase search, RRF hybrid semantic retrieval, AST call-graph tracing, and dependency mapping.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "Search term, function name, struct, or code signature." },
                        "project_name": { "type": "string", "description": "Active project identifier." }
                    },
                    "required": ["query", "project_name"]
                }
            }),
            serde_json::json!({
                "name": "orchestrate_security_audit",
                "description": "Specialist agent orchestrator for DevSecOps audits: verifies HMAC/Merkle memory integrity, scans for prompt injections, unhandled error panics, and supply chain malware.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_name": { "type": "string", "description": "Active project identifier." }
                    },
                    "required": ["project_name"]
                }
            }),
            serde_json::json!({
                "name": "orchestrate_refactor_and_fix",
                "description": "Specialist agent orchestrator for refactoring: checks historical ADR decisions, calculates failure-aware blast radius, and prepares self-healing refactoring guidelines.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_name": { "type": "string", "description": "Active project identifier." },
                        "target_symbol": { "type": "string", "description": "Function, struct, or file path to evaluate." }
                    },
                    "required": ["project_name", "target_symbol"]
                }
            }),
            serde_json::json!({
                "name": "orchestrate_memory_and_context",
                "description": "Specialist agent orchestrator for memory & token management: prunes stale AST context, manages persistent handoffs, pins invariant facts, and routes model tiers.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_name": { "type": "string", "description": "Active project identifier." },
                        "action": { "type": "string", "enum": ["load_handoff", "save_handoff", "clean_stale", "route_task"], "description": "Memory management action." }
                    },
                    "required": ["project_name", "action"]
                }
            }),
            serde_json::json!({
                "name": "orchestrate_spec_and_test",
                "description": "Specialist agent orchestrator for TDD spec generation: deconstructs raw prompts into test-driven specifications with pass/fail acceptance criteria.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "feature_request": { "type": "string", "description": "Raw user feature prompt." },
                        "project_name": { "type": "string", "description": "Target project name." }
                    },
                    "required": ["feature_request", "project_name"]
                }
            }),
            serde_json::json!({
                "name": "orchestrate_full_repo_audit",
                "description": "Master orchestrator for deep, multi-phase repository audits combining code search, security scanning, memory verification, and architectural assessment.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "repo_path": { "type": "string", "description": "Absolute path to repository." },
                        "project_name": { "type": "string", "description": "Project identifier." }
                    },
                    "required": ["repo_path", "project_name"]
                }
            }),
        ];
    }

    let mut tools = Vec::new();
    tools.extend(codebase::list_schemas());
    tools.extend(memory::list_schemas());
    tools.extend(file_ops::list_schemas());
    tools.extend(shell_git::list_schemas());
    tools.extend(refactor::list_schemas());
    tools.extend(docs::list_schemas());
    tools.extend(plugins::list_schemas());
    tools.extend(system::list_schemas());
    tools.extend(prompt_audit::list_schemas());
    tools.extend(custom_docs::list_schemas());
    tools
}

pub fn dispatch_tool_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    if let Some(res) = codebase::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = memory::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = file_ops::handle_call(name, params) {
        return Some(res);
    }
    if let Some(res) = shell_git::handle_call(name, params) {
        return Some(res);
    }
    if let Some(res) = refactor::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = docs::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = plugins::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = system::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = prompt_audit::handle_call(name, params, rt) {
        return Some(res);
    }
    if let Some(res) = custom_docs::handle_call(name, params, rt) {
        return Some(res);
    }
    None
}
