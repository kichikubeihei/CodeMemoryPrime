use serde_json::{json, Value};

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "read_file",
            "description": "Reads file contents from the local filesystem with optional line range slices.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to the file." },
                    "start_line": { "type": "integer", "description": "1-based start line (optional)." },
                    "end_line": { "type": "integer", "description": "1-based end line (optional)." }
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "write_file",
            "description": "Creates a new file or overwrites an existing file with content.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path." },
                    "content": { "type": "string", "description": "Text content to write." }
                },
                "required": ["path", "content"]
            }
        }),
        json!({
            "name": "patch_file",
            "description": "Applies a precise search-and-replace chunk edit to a target file.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to file." },
                    "target_content": { "type": "string", "description": "Exact text block to replace." },
                    "replacement_content": { "type": "string", "description": "New replacement text." }
                },
                "required": ["path", "target_content", "replacement_content"]
            }
        }),
        json!({
            "name": "list_files",
            "description": "Lists directory contents recursively or flat.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "directory_path": { "type": "string", "description": "Directory path." },
                    "recursive": { "type": "boolean", "description": "If true, walks subdirectories (default false)." }
                },
                "required": ["directory_path"]
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value) -> Option<String> {
    match name {
        "read_file" => {
            let raw_path = params.get("path").and_then(|s| s.as_str()).unwrap_or("");
            let norm_path_str = crate::path_utils::normalize_path(raw_path);
            let path_str = norm_path_str.as_str();
            let start = params.get("start_line").and_then(|s| s.as_u64()).map(|n| n as usize);
            let end = params.get("end_line").and_then(|s| s.as_u64()).map(|n| n as usize);

            match std::fs::read_to_string(path_str) {
                Ok(content) => {
                    let lines: Vec<&str> = content.lines().collect();
                    let s_idx = start.unwrap_or(1).saturating_sub(1);
                    let e_idx = end.unwrap_or(lines.len()).min(lines.len());

                    if s_idx >= lines.len() {
                        Some(format!("[Read Notice] Requested start_line {} exceeds total line count of file '{}' ({} total lines).", start.unwrap_or(1), raw_path, lines.len()))
                    } else {
                        let selected = &lines[s_idx..e_idx];
                        let mut out = format!("=== File: {} (Lines {}-{}) ===\n", raw_path, s_idx + 1, e_idx);
                        for (idx, line) in selected.iter().enumerate() {
                            out.push_str(&format!("{:4} | {}\n", s_idx + idx + 1, line));
                        }
                        Some(out)
                    }
                }
                Err(err) => Some(format!("Failed to read file '{}': {}", raw_path, err)),
            }
        }
        "write_file" => {
            let raw_path = params.get("path").and_then(|s| s.as_str()).unwrap_or("");
            let norm_path_str = crate::path_utils::normalize_path(raw_path);
            let path_str = norm_path_str.as_str();
            let content = params.get("content").and_then(|s| s.as_str()).unwrap_or("");

            if let Some(parent) = std::path::Path::new(path_str).parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            match std::fs::write(path_str, content) {
                Ok(_) => Some(format!("Successfully wrote {} bytes to '{}'.", content.len(), raw_path)),
                Err(err) => Some(format!("Failed to write file '{}': {}", raw_path, err)),
            }
        }
        "patch_file" => {
            let raw_path = params.get("path").and_then(|s| s.as_str()).unwrap_or("");
            let norm_path_str = crate::path_utils::normalize_path(raw_path);
            let path_str = norm_path_str.as_str();
            let target = params.get("target_content").and_then(|s| s.as_str()).unwrap_or("");
            let replacement = params.get("replacement_content").and_then(|s| s.as_str()).unwrap_or("");

            match std::fs::read_to_string(path_str) {
                Ok(content) => {
                    if !content.contains(target) {
                        Some(format!("Target content snippet not found in '{}'. Patch aborted.", raw_path))
                    } else {
                        let occurrences = content.matches(target).count();
                        if occurrences > 1 {
                            Some(format!("Target content matches multiple ({}) places in '{}'. Provide a more specific snippet context.", occurrences, raw_path))
                        } else {
                            let new_content = content.replace(target, replacement);
                            match std::fs::write(path_str, new_content) {
                                Ok(_) => Some(format!("Successfully patched file '{}'.", raw_path)),
                                Err(err) => Some(format!("Failed to write patched file '{}': {}", raw_path, err)),
                            }
                        }
                    }
                }
                Err(err) => Some(format!("Failed to read file '{}' for patching: {}", raw_path, err)),
            }
        }
        "list_files" => {
            let raw_dir = params.get("directory_path").and_then(|s| s.as_str()).unwrap_or("");
            let norm_dir_str = crate::path_utils::normalize_path(raw_dir);
            let dir = norm_dir_str.as_str();
            let recursive = params.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);

            let path = std::path::Path::new(dir);
            if !path.exists() {
                Some(format!("Directory '{}' does not exist.", raw_dir))
            } else {
                let mut files = Vec::new();
                if recursive {
                    let mut queue = vec![path.to_path_buf()];
                    while let Some(current) = queue.pop() {
                        if let Ok(entries) = std::fs::read_dir(&current) {
                            for entry in entries.flatten() {
                                let p = entry.path();
                                if p.is_dir() {
                                    queue.push(p);
                                } else {
                                    files.push(p.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                } else if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        files.push(entry.path().to_string_lossy().to_string());
                    }
                }

                Some(format!("Directory contents of '{}' ({} items):\n- {}", raw_dir, files.len(), files.join("\n- ")))
            }
        }
        _ => None,
    }
}
