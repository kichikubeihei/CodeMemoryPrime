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
