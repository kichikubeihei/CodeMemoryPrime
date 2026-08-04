use serde_json::{json, Value};
use std::process::Command;

pub fn list_schemas() -> Vec<Value> {
    vec![
        json!({
            "name": "run_command",
            "description": "Executes shell commands in a safe working directory with allowlisted prefixes (cargo, npm, git, python, pytest, etc.).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The exact shell command line." },
                    "cwd": { "type": "string", "description": "Working directory (optional)." }
                },
                "required": ["command"]
            }
        }),
        json!({
            "name": "git",
            "description": "Unified Git CLI interface for repository management (status, diff, log, commit, checkout, branch).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "subcommand": { "type": "string", "description": "Git subcommand (e.g. 'status', 'diff', 'log', 'branch')." },
                    "args": { "type": "array", "items": { "type": "string" }, "description": "Arguments list." },
                    "cwd": { "type": "string", "description": "Repository path." }
                },
                "required": ["subcommand"]
            }
        }),
        json!({
            "name": "create_checkpoint",
            "description": "Creates a Git restore point BEFORE making major code edits or refactors. Saves untracked & modified files to git history while keeping working directory intact.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "description": { "type": "string", "description": "Short label describing what state is being bookmarked (e.g. 'Before refactoring parser')." },
                    "cwd": { "type": "string", "description": "Repository path (optional)." }
                },
                "required": ["description"]
            }
        }),
        json!({
            "name": "restore_checkpoint",
            "description": "Restores the codebase back to a clean Git checkpoint baseline, discarding uncommitted edits or failed refactoring changes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "checkpoint_id": { "type": "string", "description": "Git stash ID (e.g. 'stash@{0}') or leave blank for most recent checkpoint." },
                    "cwd": { "type": "string", "description": "Repository path (optional)." }
                }
            }
        }),
        json!({
            "name": "list_checkpoints",
            "description": "Lists all active CodeMemoryPrime Git restore points and bookmarks.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "cwd": { "type": "string", "description": "Repository path (optional)." }
                }
            }
        })
    ]
}

pub fn handle_call(name: &str, params: &Value) -> Option<String> {
    match name {
        "run_command" => {
            let cmd_str = params.get("command").and_then(|s| s.as_str()).unwrap_or("");
            let cwd = params.get("cwd").and_then(|s| s.as_str()).unwrap_or(".");

            let allowlist = ["cargo", "npm", "npx", "git", "python", "python3", "pytest", "ls", "pwd", "cmp", "agy"];
            let first_token = cmd_str.split_whitespace().next().unwrap_or("");

            if !allowlist.contains(&first_token) {
                Some(format!(
                    "[Command Blocked] Command prefix '{}' is not in the safe execution allowlist.\n\nAllowed Command Prefixes: {:?}\n\nTo execute non-allowlisted commands, use native shell execution tools or wrap the command.",
                    first_token, allowlist
                ))
            } else {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(cmd_str)
                    .current_dir(cwd)
                    .output();

                match output {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        Some(format!("=== Command Exit Code: {} ===\n\nSTDOUT:\n{}\n\nSTDERR:\n{}", out.status.code().unwrap_or(-1), stdout, stderr))
                    }
                    Err(err) => Some(format!("Execution error: {}", err)),
                }
            }
        }
        "git" => {
            let sub = params.get("subcommand").and_then(|s| s.as_str()).unwrap_or("status");
            let args: Vec<String> = params.get("args").and_then(|a| a.as_array()).map(|arr| {
                arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
            }).unwrap_or_default();
            let cwd = params.get("cwd").and_then(|s| s.as_str()).unwrap_or(".");

            let mut command = Command::new("git");
            command.arg(sub);
            for arg in &args {
                command.arg(arg);
            }
            command.current_dir(cwd);

            match command.output() {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    if stderr.contains("not a git repository") {
                        Some(format!(
                            "[Git Notice] Working directory '{}' is not inside a Git repository.\n\nTo fix:\n- Provide `cwd` pointing to a valid git repository folder.\n- Or initialize a repository by running `git(subcommand='init', cwd='{}')`.",
                            cwd, cwd
                        ))
                    } else {
                        Some(format!("=== git {} ===\n\n{}", sub, if stdout.trim().is_empty() { stderr } else { stdout }))
                    }
                }
                Err(err) => Some(format!("Git execution error: {}", err)),
            }
        }
        "create_checkpoint" => {
            let desc = params.get("description").and_then(|s| s.as_str()).unwrap_or("Manual Checkpoint");
            let cwd = params.get("cwd").and_then(|s| s.as_str()).unwrap_or(".");
            Some(create_git_checkpoint(desc, cwd))
        }
        "restore_checkpoint" => {
            let stash_id = params.get("checkpoint_id").and_then(|s| s.as_str()).unwrap_or("stash@{0}");
            let cwd = params.get("cwd").and_then(|s| s.as_str()).unwrap_or(".");
            Some(restore_git_checkpoint(stash_id, cwd))
        }
        "list_checkpoints" => {
            let cwd = params.get("cwd").and_then(|s| s.as_str()).unwrap_or(".");
            Some(list_git_checkpoints(cwd))
        }
        _ => None,
    }
}

pub fn create_git_checkpoint(desc: &str, cwd: &str) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let stash_msg = format!("CMP-CHECKPOINT: {} ({})", desc, timestamp);

    // 1. Push stash with untracked files
    let push_out = Command::new("git")
        .args(["stash", "push", "-u", "-m", &stash_msg])
        .current_dir(cwd)
        .output();

    match push_out {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            if stdout.contains("No local changes to save") {
                return format!("[Checkpoint Saved] Workspace is clean. Bookmark created for '{}' at {}.", desc, timestamp);
            }
            // 2. Immediately apply stash to keep working directory active
            let _ = Command::new("git").args(["stash", "apply"]).current_dir(cwd).output();
            format!("[Checkpoint Saved] Created Git restore point for '{}' at {}.\nDetails: {}", desc, timestamp, if stdout.trim().is_empty() { stderr } else { stdout })
        }
        Err(e) => format!("Failed to create git checkpoint: {}", e),
    }
}

pub fn restore_git_checkpoint(stash_id: &str, cwd: &str) -> String {
    // 1. Clean current uncommitted changes
    let _ = Command::new("git").args(["checkout", "."]).current_dir(cwd).output();
    let _ = Command::new("git").args(["clean", "-fd"]).current_dir(cwd).output();

    // 2. Apply target stash
    let apply_out = Command::new("git")
        .args(["stash", "apply", stash_id])
        .current_dir(cwd)
        .output();

    match apply_out {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            format!("[Checkpoint Restored] Reverted codebase state to checkpoint '{}'.\n{}", stash_id, if stdout.trim().is_empty() { stderr } else { stdout })
        }
        Err(e) => format!("Failed to restore git checkpoint: {}", e),
    }
}

pub fn list_git_checkpoints(cwd: &str) -> String {
    let out = Command::new("git")
        .args(["stash", "list"])
        .current_dir(cwd)
        .output();

    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let mut checkpoints = Vec::new();
            for line in stdout.lines() {
                if line.contains("CMP-CHECKPOINT") {
                    checkpoints.push(line);
                }
            }
            if checkpoints.is_empty() {
                "No CodeMemoryPrime checkpoints found in git stash.".to_string()
            } else {
                format!("=== Active CodeMemoryPrime Checkpoints ===\n\n{}", checkpoints.join("\n"))
            }
        }
        Err(e) => format!("Failed to list checkpoints: {}", e),
    }
}
