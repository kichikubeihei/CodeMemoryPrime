use std::io::{self, BufRead, Write};
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use tracing::info;
use codememory_prime::protocol::handlers::{RpcRequest, handle_request};

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
        
    info!("Starting CodeMemoryPrime (CMP) server");

    let rt = match Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[CMP Error] Failed to initialize async runtime: {}", e);
            return;
        }
    };

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "preflight" | "--preflight" | "cmp_preflight" => {
                println!("{}", codememory_prime::tools::run_preflight_audit(&rt));
                return;
            }
            "sync" | "--sync" | "sync_memory" | "sync-memory" => {
                let ep = args.get(2).map(|s| s.as_str());
                println!("{}", codememory_prime::tools::run_sync_memory(ep, &rt));
                return;
            }
            "research" | "--research" => {
                let keyword = args.get(2).map(|s| s.as_str()).unwrap_or("");
                let db_path = codememory_prime::get_db_path();
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    match codememory_prime::research_vault::query_research(&conn, "all", keyword, 10) {
                        Ok(recs) => {
                            if recs.is_empty() {
                                println!("No research records found.");
                            } else {
                                println!("=== Research & Video Records ({}) ===", recs.len());
                                for r in recs {
                                    println!("\n• [{}] {}", r.title, r.media_url);
                                    println!("  Project: {} | Type: {} | Date: {}", r.target_project, r.media_type, r.created_at);
                                    println!("  Takeaways: {}", r.key_takeaways);
                                    println!("  Upgrades:  {}", r.proposed_upgrades);
                                }
                            }
                        }
                        Err(e) => eprintln!("Error querying research: {}", e),
                    }
                }
                return;
            }
            "--help" | "-h" => {
                println!("CodeMemoryPrime (CMP) - Unified Codebase Memory & Intelligence Engine");
                println!("\nUsage:");
                println!("  cmp                      Start MCP stdio JSON-RPC server (default)");
                println!("  cmp preflight            Run full system preflight audit");
                println!("  cmp sync-memory [URL]    Sync GPU roster and export local delta snapshot");
                println!("  cmp research [KEYWORD]   Query recorded video analyses & architecture upgrades");
                println!("\nNote: Automated multi-device cloud sync (Cloudflare R2, Tailscale daemon) is in CodeMemoryPrime-Pro (`cmp-pro`).");
                return;
            }
            _ => {}
        }
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    for line in reader.lines() {
        if let Ok(line_str) = line {
            if line_str.trim().is_empty() { continue; }
            if let Ok(req) = serde_json::from_str::<RpcRequest>(&line_str) {
                // If the request has no 'id', it is a Notification per JSON-RPC 2.0 spec. Do NOT reply.
                if req.id.is_none() || req.id == Some(Value::Null) {
                    let _ = handle_request(req, &rt);
                    continue;
                }

                let id = req.id.clone().unwrap_or(Value::Null);
                let result = handle_request(req, &rt);
                
                let is_error = result.get("error").is_some();
                let res = if is_error {
                    let err_obj = result.get("error").cloned().unwrap_or_else(|| json!({"code": -32601, "message": "Method not found"}));
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": if err_obj.is_object() { err_obj } else { json!({"code": -32603, "message": err_obj.to_string()}) }
                    })
                } else {
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result
                    })
                };

                if let Ok(out) = serde_json::to_string(&res) {
                    let _ = writeln!(stdout, "{}", out);
                    let _ = stdout.flush();
                }
            }
        }
    }
}
