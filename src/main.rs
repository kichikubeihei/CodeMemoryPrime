mod db;
mod llm;
mod parser;
mod search;
mod scraper;
mod watcher;
mod license;
mod protocol;
mod tools;

use std::io::{self, BufRead, Write};
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use tracing::info;
use protocol::handlers::{RpcRequest, handle_request};

pub fn get_db_path() -> String {
    let base = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let new_path = format!("{}/.codememory_prime.db", base);
    let old_path = format!("{}/.coder_memory.db", base);
    if !std::path::Path::new(&new_path).exists() && std::path::Path::new(&old_path).exists() {
        old_path
    } else {
        new_path
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();
        
    info!("Starting CodeMemoryPrime (CMP) server");

    let rt = Runtime::new().unwrap();
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

                let id = req.id.clone().unwrap();
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

                let out = serde_json::to_string(&res).unwrap();
                writeln!(stdout, "{}", out).unwrap();
                stdout.flush().unwrap();
            }
        }
    }
}
