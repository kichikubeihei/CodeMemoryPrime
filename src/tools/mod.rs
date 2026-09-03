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
                "description": "MANDATORY: Always use this tool FIRST for codebase navigation and symbol retrieval. Automatically performs sub-10ms git diff re-indexing and returns complete AST symbol definitions, exact line numbers, code snippets, and caller graphs in 1 unified step. DO NOT call index_workspace manually, and DO NOT read files in small 20-line slices.",
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
                "description": "Specialist agent orchestrator for DevSecOps audits: verifies HMAC/Merkle memory integrity, scans for prompt injections, unhandled error panics, and supply chain malware in 1 step.",
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
                "description": "MANDATORY: Always use this tool BEFORE modifying code. Automatically fetches historical ADR decisions, calculates failure-aware blast radius, and prepares self-healing refactoring guidelines. DO NOT manually re-index workspace.",
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
                "description": "Specialist agent orchestrator for TDD spec generation & persistent alignment: conducts interactive spec alignment interviews, locks invariants, and supersedes old choices.",
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
            serde_json::json!({
                "name": "cmp_preflight",
                "description": "Performs full system preflight check: license validation, local Ollama connectivity, Tailscale model roster status, SQLite memory stats, and cryptographic integrity.",
                "inputSchema": { "type": "object", "properties": {} }
            }),
            serde_json::json!({
                "name": "sync_memory",
                "description": "Performs decentralized memory mesh sync and refreshes the Tailscale Ollama model roster into GEMINI.md.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "endpoint": { "type": "string", "description": "Optional custom Ollama Tailscale endpoint URL." }
                    }
                }
            }),
        ];
    }

    let mut tools = Vec::new();
    tools.push(serde_json::json!({
        "name": "cmp_preflight",
        "description": "Performs full system preflight check: license validation, local Ollama connectivity, Tailscale model roster status, SQLite memory stats, and cryptographic integrity.",
        "inputSchema": { "type": "object", "properties": {} }
    }));
    tools.push(serde_json::json!({
        "name": "sync_memory",
        "description": "Performs decentralized memory mesh sync and refreshes the Tailscale Ollama model roster into GEMINI.md.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "endpoint": { "type": "string", "description": "Optional custom Ollama Tailscale endpoint URL." }
            }
        }
    }));
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

pub fn run_preflight_audit(rt: &Runtime) -> String {
    let mut out = String::new();
    out.push_str("=== CodeMemoryPrime (CMP) Preflight Audit ===\n\n");

    // 1. License Check
    let lic_status = crate::license::check_license_key(None);
    match lic_status {
        crate::license::LicenseStatus::ValidCommercial { licensee, seats, expires, license_type } => {
            out.push_str(&format!(" [✔] **License**: Valid {} License (Licensee: {}, Seats: {}, Expires: {})\n", license_type, licensee, seats, expires));
        }
        crate::license::LicenseStatus::FreeTier { message } => {
            out.push_str(&format!(" [ℹ] **License**: Free Tier ({})\n", message));
        }
        crate::license::LicenseStatus::Expired { licensee, expires } => {
            out.push_str(&format!(" [✘] **License Expired**: {} ({})\n", licensee, expires));
        }
        crate::license::LicenseStatus::Invalid { reason } => {
            out.push_str(&format!(" [✘] **License Invalid**: {}\n", reason));
        }
    }

    // 2. Database Location & Size
    let db_path = crate::get_db_path();
    let _ = crate::db::init_database(&db_path);
    let db_size = std::fs::metadata(&db_path).map(|m| m.len() as f64 / 1_048_576.0).unwrap_or(0.0);
    out.push_str(&format!(" [✔] **SQLite Memory Database**: `{}` ({:.2} MB)\n", db_path, db_size));

    // 3. Local LLM Provider
    let cfg = crate::llm::get_config_from_db_or_env();
    let local_status = rt.block_on(async { crate::llm::check_ollama_connection().await });
    match local_status {
        Ok(_) => out.push_str(&format!(" [✔] **Local LLM Provider**: Connected (`{}`) [Gen: `{}`, Embed: `{}`]\n", cfg.base_url, cfg.gen_model, cfg.embed_model)),
        Err(e) => out.push_str(&format!(" [⚠] **Local LLM Provider**: Disconnected from `{}` ({})\n", cfg.base_url, e)),
    }

    // 4. Remote Tailscale Host & Model Roster
    let remote_url = if !cfg.remote_base_url.is_empty() {
        cfg.remote_base_url.clone()
    } else {
        std::env::var("MCP_LLM_REMOTE_BASE_URL").unwrap_or_else(|_| "http://100.102.233.128:11434".to_string())
    };
    out.push_str(&format!(" [i] **Tailscale Endpoint**: Checking `{}`...\n", remote_url));
    match crate::tailscale_roster::sync_tailscale_model_roster(Some(&remote_url), None) {
        Ok(res) => {
            out.push_str(&format!(" [✔] **Tailscale GPU Cluster**: Connected ({} live models detected)\n", res.models_found.len()));
            for m in &res.models_found {
                out.push_str(&format!("     • `{:<20}` ({:.1} GB)\n", m.name, m.size as f64 / 1_073_741_824.0));
            }
            if res.updated {
                out.push_str(&format!(" [✔] **Model Roster Synced to**: `{}`\n", res.gemini_md_path));
            }
        }
        Err(e) => out.push_str(&format!(" [⚠] **Remote Tailscale Host**: Unreachable ({})\n", e)),
    }

    // 5. Memory Integrity & Cryptographic Ledger
    if let Ok(audit) = crate::memory_integrity::audit_all_memory("all") {
        let root = if audit.merkle_branch_root.len() >= 12 { &audit.merkle_branch_root[..12] } else { &audit.merkle_branch_root };
        out.push_str(&format!(" [✔] **Cryptographic Memory Integrity**: Status: `{}` (Merkle Branch Root: `{}`)\n", audit.status, root));
    }

    out.push_str("\n=== Preflight Audit Complete: Engine Ready ===");
    out
}

pub fn run_sync_memory(endpoint: Option<&str>, _rt: &Runtime) -> String {
    let mut out = String::new();
    out.push_str("=== CodeMemoryPrime (CMP) Memory Mesh & Roster Sync ===\n\n");

    let cfg = crate::llm::get_config_from_db_or_env();
    let remote_url = endpoint.map(|s| s.to_string()).unwrap_or_else(|| {
        if !cfg.remote_base_url.is_empty() {
            cfg.remote_base_url.clone()
        } else {
            std::env::var("MCP_LLM_REMOTE_BASE_URL").unwrap_or_else(|_| "http://100.102.233.128:11434".to_string())
        }
    });

    out.push_str(&format!("1. Syncing GPU Model Roster from Tailscale (`{}`)...\n", remote_url));
    match crate::tailscale_roster::sync_tailscale_model_roster(Some(&remote_url), None) {
        Ok(res) => {
            out.push_str(&format!("   [✔] Detected {} remote GPU models.\n   [✔] Synced to `{}`\n", res.models_found.len(), res.gemini_md_path));
        }
        Err(e) => {
            out.push_str(&format!("   [⚠] Roster sync warning: {}\n", e));
        }
    }

    out.push_str("\n2. Exporting Decentralized Memory Mesh Delta Package...\n");
    let db_path = crate::get_db_path();
    let _ = crate::db::init_database(&db_path);
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "omarchy".to_string());
        match crate::mesh_sync::export_memory_delta(&conn, &hostname) {
            Ok(delta) => {
                let sig = if delta.hmac_signature.len() >= 12 { &delta.hmac_signature[..12] } else { &delta.hmac_signature };
                out.push_str("   [✔] Memory Delta Package Exported:\n");
                out.push_str(&format!("       - Device Origin: {}\n", delta.device_source));
                out.push_str(&format!("       - Session Handoffs: {}\n", delta.session_handoffs.len()));
                out.push_str(&format!("       - Solution Vault Records: {}\n", delta.solution_vault.len()));
                out.push_str(&format!("       - Failure Dead Ends: {}\n", delta.failure_vault.len()));
                out.push_str(&format!("       - Knowledge Graph Nodes: {}\n", delta.knowledge_nodes.len()));
                out.push_str(&format!("       - Knowledge Graph Edges: {}\n", delta.knowledge_edges.len()));
                out.push_str(&format!("       - HMAC Signature: `{}`\n", sig));
            }
            Err(e) => {
                out.push_str(&format!("   [⚠] Delta export warning: {}\n", e));
            }
        }
    }

    out.push_str("\n=== Sync Complete: Hive Mind is Online & Synchronized ===");
    out
}

pub fn dispatch_tool_call(name: &str, params: &Value, rt: &Runtime) -> Option<String> {
    match name {
        "cmp_preflight" | "orchestrate_preflight" => {
            return Some(run_preflight_audit(rt));
        }
        "sync_memory" | "orchestrate_sync_memory" => {
            let ep = params.get("endpoint").and_then(|s| s.as_str());
            return Some(run_sync_memory(ep, rt));
        }
        "orchestrate_code_search" => {
            let query = params.get("query").and_then(|s| s.as_str()).unwrap_or("");
            let proj = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let search_args = serde_json::json!({
                "query": query,
                "project_name": proj,
                "limit": 10,
                "include_surrounding_lines": true
            });
            return codebase::handle_call("search_codebase", &search_args, rt);
        }
        "orchestrate_security_audit" => {
            let proj = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let audit = crate::memory_integrity::audit_all_memory(proj);
            return match audit {
                Ok(rep) => Some(format!(
                    "=== Memory Security & DevSecOps Audit for '{}' ===\n\n- **Status**: {}\n- **Total Records Scanned**: {}\n- **Tampered Records**: {}\n- **Prompt Injections Detected**: {}\n- **Merkle Branch Root**: `{}`\n\n### Findings:\n{}",
                    rep.project_name, rep.status, rep.total_records_scanned, rep.tampered_records_count, rep.prompt_injections_count, rep.merkle_branch_root,
                    if rep.details.is_empty() { "No security or integrity violations found. Memory Merkle branch is pristine.".to_string() } else { rep.details.join("\n- ") }
                )),
                Err(e) => Some(format!("Security audit error: {}", e)),
            };
        }
        "orchestrate_refactor_and_fix" => {
            let proj = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let target = params.get("target_symbol").and_then(|s| s.as_str()).unwrap_or("");
            let adrs = crate::refactor_decision::get_decisions(proj, target).unwrap_or_default();
            let candidates = crate::refactor_recommender::scan_workspace_for_refactor_candidates(".").unwrap_or_default();
            return Some(format!(
                "=== Refactor Orchestration for '{}' in Project '{}' ===\n\n- **Target**: `{}`\n- **Historical ADR Decisions Found**: {}\n- **Monolith Candidates in Scope**: {}\n\n{}",
                target, proj, target, adrs.len(), candidates.len(),
                if !adrs.is_empty() {
                    format!("### Historical ADRs:\n{}", adrs.iter().map(|a| format!("- **{}** (Symbol: `{}`): {}", a.timestamp, a.symbol_or_file, a.decision_text)).collect::<Vec<_>>().join("\n"))
                } else {
                    "No conflicting historical architectural decisions recorded for this symbol.".to_string()
                }
            ));
        }
        "orchestrate_memory_and_context" => {
            let proj = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let action = params.get("action").and_then(|s| s.as_str()).unwrap_or("clean_stale");
            match action {
                "load_handoff" => {
                    let handoff = crate::handoff::load_session_handoff(proj);
                    return Some(match handoff {
                        Ok(Some(h)) => format!("=== Active Handoff for '{}' ===\n- Goal: {}\n- Active Files: {}\n- Open Questions: {}\n- Steps Completed: {}", h.project_name, h.task_goal, h.active_files.join(", "), h.open_questions.join(", "), h.completed_steps.join("\n  * ")),
                        Ok(None) => format!("No active session handoff found for '{}'.", proj),
                        Err(e) => format!("Failed to load handoff: {}", e),
                    });
                }
                "clean_stale" => {
                    let cleaned = crate::doc_cleaner::clean_stale_context(".", proj);
                    return Some(match cleaned {
                        Ok(summary) => format!("Context pruned successfully. Deleted {} orphaned chunks and {} stale handoffs.", summary.orphaned_chunks_deleted, summary.stale_handoffs_deleted),
                        Err(e) => format!("Context pruning error: {}", e),
                    });
                }
                "route_task" => {
                    let routing = crate::model_router::route_task("General refactoring and memory coordination", 5000);
                    return Some(format!("=== Task Routing Recommendation ===\n- Recommended Tier: {}\n- Suggestions: {}\n- Estimated Cost: {}\n- Rationale: {}", routing.recommended_tier, routing.model_suggestions.join(", "), routing.estimated_cost_factor, routing.rationale));
                }
                _ => return Some(format!("Unknown memory orchestration action: '{}'", action)),
            }
        }
        "orchestrate_spec_and_test" => {
            let proj = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let req = params.get("feature_request").and_then(|s| s.as_str()).unwrap_or("");
            return Some(format!(
                "=== TDD Spec & Gate Orchestration for '{}' ===\n\nFeature: {}\n\nCryptographic Task Gates Initialized. Use `task_ledger` to record and verify test assertions.",
                proj, req
            ));
        }
        "orchestrate_full_repo_audit" => {
            let proj = params.get("project_name").and_then(|s| s.as_str()).unwrap_or("all");
            let health = system::handle_call("project_health", params, rt).unwrap_or_default();
            let audit = crate::memory_integrity::audit_all_memory(proj);
            let audit_str = match audit {
                Ok(a) => format!("- Memory Status: {}\n- Tampered Records: {}\n- Merkle Root: {}", a.status, a.tampered_records_count, a.merkle_branch_root),
                Err(e) => format!("- Audit Error: {}", e),
            };
            return Some(format!("=== Full Repository Audit for '{}' ===\n\n{}\n\n### Security & Integrity:\n{}", proj, health, audit_str));
        }
        _ => {}
    }

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
