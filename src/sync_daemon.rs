use crate::mesh_sync::{export_memory_delta, import_memory_delta, MemoryDeltaPackage};
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

pub async fn run_sync_daemon(port: u16) -> Result<(), String> {
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind sync daemon to {}: {}", addr, e))?;

    println!("=====================================================");
    println!("  CodeMemoryPrime Memory Sync Daemon (CMP-Mesh)");
    println!("  Listening on: http://{}", addr);
    println!("  Endpoints:");
    println!("    • POST /api/memory/delta  (Push delta package)");
    println!("    • GET  /api/memory/deltas (Pull delta packages)");
    println!("    • GET  /api/memory/health (Health check)");
    println!("=====================================================");

    let memory_pool = Arc::new(Mutex::new(Vec::<MemoryDeltaPackage>::new()));

    loop {
        let (mut socket, _peer) = match listener.accept().await {
            Ok(conn) => conn,
            Err(_) => continue,
        };

        let pool = Arc::clone(&memory_pool);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024 * 1024]; // 1MB buffer
            let n = match socket.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => return,
            };

            let req_str = String::from_utf8_lossy(&buf[..n]);
            let mut lines = req_str.lines();
            let req_line = lines.next().unwrap_or("");
            let parts: Vec<&str> = req_line.split_whitespace().collect();

            if parts.len() < 2 {
                return;
            }

            let method = parts[0];
            let path = parts[1];

            // Extract body (after \r\n\r\n or \n\n)
            let body = if let Some(pos) = req_str.find("\r\n\r\n") {
                &req_str[pos + 4..]
            } else if let Some(pos) = req_str.find("\n\n") {
                &req_str[pos + 2..]
            } else {
                ""
            };

            let (status_line, content_type, response_body) = if method == "GET" && path == "/api/memory/health" {
                ("HTTP/1.1 200 OK", "application/json", r#"{"status":"ok","engine":"CodeMemoryPrime"}"#.to_string())
            } else if method == "GET" && path == "/api/memory/deltas" {
                let db_path = crate::get_db_path();
                let mut all = Vec::new();
                if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                    if let Ok(local) = export_memory_delta(&conn, "daemon") {
                        all.push(local);
                    }
                }
                if let Ok(lock) = pool.lock() {
                    for item in lock.iter() {
                        all.push(item.clone());
                    }
                }
                let body = serde_json::to_string(&all).unwrap_or_else(|_| "[]".to_string());
                ("HTTP/1.1 200 OK", "application/json", body)
            } else if method == "POST" && path == "/api/memory/delta" {
                match serde_json::from_str::<MemoryDeltaPackage>(body) {
                    Ok(pkg) => {
                        let db_path = crate::get_db_path();
                        let mut merged = 0;
                        if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                            if let Ok(rep) = import_memory_delta(&conn, &pkg) {
                                merged = rep.handoffs_merged + rep.research_merged + rep.solutions_merged;
                            }
                        }
                        if let Ok(mut lock) = pool.lock() {
                            lock.push(pkg);
                            if lock.len() > 50 {
                                lock.remove(0);
                            }
                        }
                        let resp = format!(r#"{{"status":"success","merged":{}}}"#, merged);
                        ("HTTP/1.1 200 OK", "application/json", resp)
                    }
                    Err(e) => {
                        let resp = format!(r#"{{"error":"Invalid delta package: {}"}}"#, e);
                        ("HTTP/1.1 400 Bad Request", "application/json", resp)
                    }
                }
            } else {
                ("HTTP/1.1 404 Not Found", "text/plain", "Not Found".to_string())
            };

            let resp = format!(
                "{}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                status_line,
                content_type,
                response_body.len(),
                response_body
            );

            let _ = socket.write_all(resp.as_bytes()).await;
        });
    }
}
