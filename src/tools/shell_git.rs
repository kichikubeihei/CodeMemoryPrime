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
        _ => None,
    }
}
